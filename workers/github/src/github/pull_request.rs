use crate::dto::pull_request::PullRequestsDTO;
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
struct PullRequestContributionsQuery;

pub async fn get_user_pull_requests(token: String, username: String) -> Result<()> {
    info!("Token passed -> {}", token);
    info!("Username passed -> {}", username);

    let request_body =
        PullRequestContributionsQuery::build_query(pull_request_contributions_query::Variables {
            username: username.clone(),
        });
    let mut raw_response = reqwest::Client::new()
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&request_body)
        .send()?;

    let response: Response<pull_request_contributions_query::ResponseData> = raw_response
        .json()
        .context("Attempting to deserialize the response object")?;

    if let Some(errors) = response.errors {
        error!("Got errors from querying the github API for contributions");

        for err in errors {
            error!("{:#?}", err);
            info!("{:#?}", err);
        }
    }

    let pull_request_contributions_data: pull_request_contributions_query::ResponseData = response
        .data
        .context("Retrieving the pull request contribution's response data")?;
    if let Some(user) = pull_request_contributions_data.user {
        if let Some(nodes) = user
            .contributions_collection
            .pull_request_contributions
            .nodes
        {
            debug!("{:?}", nodes);
        }
    }
    Ok(())
}
