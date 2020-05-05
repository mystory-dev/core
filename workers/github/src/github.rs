use anyhow::*;
use graphql_client::{GraphQLQuery, Response};
use log::{debug, error, info, trace};

type Date = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github_schema.graphql",
    query_path = "schemas/queries.graphql",
    response_derives = "Debug"
)]
struct ContributionsQuery;

type Contribution = (Date, i64);

#[derive(Debug)]
pub struct ContributionsDTO {
    pub contributions: Vec<Contribution>,
}

impl ContributionsDTO {
    fn new() -> ContributionsDTO {
        ContributionsDTO {
            contributions: Vec::new(),
        }
    }

    fn add_contribution(&mut self, contribution: Contribution) {
        self.contributions.push(contribution)
    }
}

pub async fn get_user_contributions(token: String, username: String) -> Result<ContributionsDTO> {
    info!("Token passed -> {}", token);
    info!("Username passed -> {}", username);

    let request_body = ContributionsQuery::build_query(contributions_query::Variables {
        username: username.clone(),
    });
    let mut raw_response = reqwest::Client::new()
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&request_body)
        .send()?;

    let response: Response<contributions_query::ResponseData> = raw_response
        .json()
        .context("Attempting to deserialize the response object")?;

    if let Some(errors) = response.errors {
        error!("Got errors from querying the github API for contributions");

        for err in errors {
            error!("{:#?}", err);
        }
    }

    let contributions_data: contributions_query::ResponseData = response
        .data
        .context("Retrieving the contribution's response data")?;

    if let Some(user) = contributions_data.user {
        let mut contributions = ContributionsDTO::new();

        for week in user.contributions_collection.contribution_calendar.weeks {
            for day in week.contribution_days {
                contributions.add_contribution((day.date, day.contribution_count))
            }
        }

        info!(
            "For [{}] we were able to retrieve {} contributions",
            &username,
            contributions.contributions.len()
        );
        return Ok(contributions);
    };

    Err(anyhow!("Failed to retrieve a user from the github API"))
}
