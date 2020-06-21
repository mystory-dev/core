use crate::dto::pull_requests::PullRequestsDTO;
use anyhow::*;
use graphql_client::{GraphQLQuery, Response};
use log::{debug, error};

type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github_schema.graphql",
    query_path = "schemas/queries.graphql",
    response_derives = "Debug"
)]
struct PullRequestReviewsQuery;

async fn make_graphql_call(
    name: String,
    owner: String,
    pull_request_number: i64,
    next_cursor: &Option<String>,
    token: String,
) -> Result<Response<pull_request_reviews_query::ResponseData>> {
    let request_body =
        PullRequestReviewsQuery::build_query(pull_request_reviews_query::Variables {
            name: String::from(name),
            owner: String::from(owner),
            number: pull_request_number,
            current_cursor: next_cursor.clone(),
        });
    let mut raw_response = reqwest::Client::new()
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&request_body)
        .send()?;

    Ok(raw_response
        .json()
        .context("Attempting to deserialize the response object")?)
}

pub async fn fetch_pull_request_reviews(
    name_with_owner: String,
    pull_request_number: i64,
    pull_request_id: &String,
    token: String,
    pull_request_dto: &mut PullRequestsDTO,
) -> Result<()> {
    debug!(
        "Branching off to fetch all reviews for the pull request {}",
        pull_request_id
    );
    let name_tokens: Vec<&str> = name_with_owner.split("/").collect();
    let mut has_more_reviews = false;
    let mut next_cursor: Option<String> = None;

    loop {
        debug!(
            "Fetching the next reviews for pull request -> {}...",
            pull_request_id
        );
        let response: Response<pull_request_reviews_query::ResponseData> = make_graphql_call(
            String::from(name_tokens[1]),
            String::from(name_tokens[0]),
            pull_request_number,
            &next_cursor,
            token.clone(),
        )
        .await?;

        if let Some(errors) = response.errors {
            error!("Got errors from querying the github API for contributions");

            for err in errors {
                error!("{:#?}", err);
            }
            break;
        }

        let reviews_response: pull_request_reviews_query::ResponseData = response
            .data
            .context("Retrieving the pull request contribution's response data")?;

        if let Some(repository) = reviews_response.repository {
            if let Some(pull_request) = repository.pull_request {
                if let Some(reviews) = pull_request.reviews {
                    if reviews.page_info.has_next_page {
                        if let Some(end_cursor) = reviews.page_info.end_cursor {
                            has_more_reviews = reviews.page_info.has_next_page;
                            next_cursor = Some(end_cursor);
                        }
                    }
                    if let Some(reviews_collection) = reviews.nodes {
                        for review in reviews_collection {
                            if let Some(review) = review {
                                let review_state: String = String::from(match review.state {
                                    pull_request_reviews_query::PullRequestReviewState::APPROVED => "Approved",
                                    pull_request_reviews_query::PullRequestReviewState::CHANGES_REQUESTED => "Changes requested",
                                    pull_request_reviews_query::PullRequestReviewState::DISMISSED => "Dismissed",
                                    pull_request_reviews_query::PullRequestReviewState::COMMENTED => "Commented",
                                    _ => "Pending",
                                });

                                if let Some(published_at) = review.published_at {
                                    pull_request_dto.add_review(
                                        pull_request_id,
                                        review.id,
                                        published_at,
                                        review.viewer_did_author,
                                        review_state,
                                        String::from("Something"), //review.author.unwrap().on.user.id,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if !has_more_reviews {
            break;
        }
    }

    Ok(())
}
