use crate::dto::pull_requests::PullRequestsDTO;
use anyhow::*;
use graphql_client::{GraphQLQuery, Response};
use log::{debug, error, info};

type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github_schema.graphql",
    query_path = "schemas/queries.graphql",
    response_derives = "Debug"
)]
struct PullRequestCommitsQuery;

async fn make_graphql_call(
    name: String,
    owner: String,
    pull_request_number: i64,
    next_cursor: &Option<String>,
    token: String,
) -> Result<Response<pull_request_commits_query::ResponseData>> {
    let request_body =
        PullRequestCommitsQuery::build_query(pull_request_commits_query::Variables {
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

pub async fn fetch_pull_request_commits(
    name_with_owner: String,
    pull_request_number: i64,
    pull_request_id: &String,
    token: String,
    pull_request_dto: &mut PullRequestsDTO,
) -> Result<()> {
    debug!(
        "Branching off to fetch all commits for the pull request -> {}",
        pull_request_id
    );
    let name_tokens: Vec<&str> = name_with_owner.split("/").collect();
    let mut has_more_commits = false;
    let mut next_cursor: Option<String> = None;

    loop {
        debug!(
            "Fetching the next 100 commits for pull request -> {}",
            pull_request_id
        );
        let response: Response<pull_request_commits_query::ResponseData> = make_graphql_call(
            String::from(name_tokens[1]),
            String::from(name_tokens[0]),
            pull_request_number,
            &next_cursor,
            token.clone(),
        )
        .await?;

        if let Some(errors) = response.errors {
            error!(
                "Got errors from querying the github API for commits on the pull request -> {}",
                pull_request_id
            );

            for err in errors {
                error!("{:#?}", err);
            }
            break;
        }

        let commits_response: pull_request_commits_query::ResponseData =
            response.data.context(format!(
                "Serializing commit data for the pull request -> {}",
                pull_request_id
            ))?;

        if let Some(repository) = commits_response.repository {
            if let Some(pull_request) = repository.pull_request {
                if let Some(end_cursor) = pull_request.commits.page_info.end_cursor {
                    if pull_request.commits.page_info.has_next_page {
                        has_more_commits = true;
                        next_cursor = Some(end_cursor);
                    }
                }

                for commit_collection in pull_request.commits.nodes {
                    for commit in commit_collection {
                        if let Some(commit) = commit {
                            if let Some(pushed_date) = commit.commit.pushed_date {
                                pull_request_dto.add_commit(
                                    &pull_request_id,
                                    commit.id,
                                    pushed_date,
                                    commit.commit.abbreviated_oid,
                                    commit.commit.author.unwrap().user.unwrap().id,
                                );
                            }
                        }
                    }
                }
            }
        }

        if !has_more_commits {
            break;
        }
    }

    Ok(())
}
