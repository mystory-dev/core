use crate::dto::PullRequestsDTO;
use crate::github::commits::fetch_pull_request_commits;
use crate::github::reviews::fetch_pull_request_reviews;
use anyhow::*;
use graphql_client::GraphQLQuery;
use graphql_client::Response;
use log::{debug, error};

type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github_schema.graphql",
    query_path = "schemas/queries.graphql",
    response_derives = "Debug"
)]
struct PullRequestReviewContributionsQuery;

pub async fn fetch_pull_requests(
    token: String,
    username: String,
    current_cursor: String,
) -> Result<pull_request_review_contributions_query::ResponseData> {
    let request_body = PullRequestReviewContributionsQuery::build_query(
        pull_request_review_contributions_query::Variables {
            username: username.clone(),
            current_cursor: Some(current_cursor.clone()),
        },
    );
    let mut raw_response = reqwest::Client::new()
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&request_body)
        .send()?;

    let response: Response<pull_request_review_contributions_query::ResponseData> = raw_response
        .json()
        .context("Attempting to deserialize the response object")?;

    if let Some(errors) = response.errors {
        error!("Got errors from querying the github API for contributions");

        for err in errors {
            error!("{:#?}", err);
        }
    }

    Ok(response
        .data
        .context("Retrieving the pull request contribution's response data")?)
}

pub async fn get_pull_request_review_contributions(
    token: String,
    username: String,
    mut pull_request_dto: &mut PullRequestsDTO,
) -> Result<&PullRequestsDTO> {
    let mut current_cursor: String = String::from("");

    loop {
        debug!("Taking the next 100 pull request review contributions...");
        let pull_request_review_contributions_data =
            fetch_pull_requests(token.clone(), username.clone(), current_cursor.clone()).await?;

        if let Some(user) = pull_request_review_contributions_data.user {
            if let Some(nodes) = user
                .contributions_collection
                .pull_request_review_contributions
                .nodes
            {
                for contribution in nodes {
                    if let Some(contribution) = contribution {
                        if let pull_request_review_contributions_query::PullRequestReviewContributionsQueryUserContributionsCollectionPullRequestReviewContributionsNodesPullRequestAuthorOn::User(author) = contribution.pull_request.author.unwrap().on {

                            pull_request_dto.add_pull_request(
                                contribution.pull_request.id.clone(),
                                contribution.pull_request.created_at,
                                contribution.pull_request.closed_at,
                                author.id,
                                contribution.pull_request.number.clone(),
                            );

                        }

                        let repository = contribution.pull_request.repository;

                        if let Some(reviews) = contribution.pull_request.reviews {
                            if reviews.page_info.has_next_page {
                                fetch_pull_request_reviews(
                                    repository.name_with_owner.clone(),
                                    contribution.pull_request.number.clone(),
                                    &contribution.pull_request.id.clone(),
                                    token.clone(),
                                    &mut pull_request_dto,
                                )
                                .await?;
                            } else {
                                for review_collection in reviews.nodes {
                                    for review in review_collection {
                                        if let Some(review) = review {
                                            let review_state: String = String::from(match review.state {
                                                    pull_request_review_contributions_query::PullRequestReviewState::APPROVED => "Approved",
                                                    pull_request_review_contributions_query::PullRequestReviewState::CHANGES_REQUESTED => "Changes requested",
                                                    pull_request_review_contributions_query::PullRequestReviewState::DISMISSED => "Dismissed",
                                                    pull_request_review_contributions_query::PullRequestReviewState::COMMENTED => "Commented",
                                                    _ => "Pending",
                                                });

                                            if let Some(published_at) = review.published_at {
                                                if let pull_request_review_contributions_query::PullRequestReviewContributionsQueryUserContributionsCollectionPullRequestReviewContributionsNodesPullRequestReviewsNodesAuthorOn::User(author) = review.author.unwrap().on {
                                                        pull_request_dto.add_review(
                                                            &contribution.pull_request.id,
                                                            review.id,
                                                            published_at,
                                                            review.viewer_did_author,
                                                            review_state,
                                                            author.id,
                                                        );
                                                    }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if contribution.pull_request.commits.page_info.has_next_page {
                            fetch_pull_request_commits(
                                repository.name_with_owner.clone(),
                                contribution.pull_request.number.clone(),
                                &contribution.pull_request.id.clone(),
                                token.clone(),
                                &mut pull_request_dto,
                            )
                            .await?;
                        } else {
                            for commit_collection in contribution.pull_request.commits.nodes {
                                for commit in commit_collection {
                                    if let Some(commit) = commit {
                                        if let Some(pushed_date) = commit.commit.pushed_date {
                                            let mut author_id: String = String::from("");

                                            if let Some(author) = commit.commit.author {
                                                if let Some(user) = author.user {
                                                    author_id = user.id;
                                                }
                                            }

                                            pull_request_dto.add_commit(
                                                &contribution.pull_request.id,
                                                commit.id,
                                                pushed_date,
                                                commit.commit.abbreviated_oid,
                                                author_id,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if user
                .contributions_collection
                .pull_request_review_contributions
                .page_info
                .has_next_page
            {
                if let Some(end_cursor) = user
                    .contributions_collection
                    .pull_request_review_contributions
                    .page_info
                    .end_cursor
                {
                    current_cursor = end_cursor;
                    continue;
                }
            }

            break;
        }
    }

    Ok(pull_request_dto)
}
