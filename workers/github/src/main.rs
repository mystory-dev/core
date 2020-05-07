use crate::store::{Store, StoreDestination};
use anyhow::*;
use dotenv::dotenv;
use env_logger::{Builder, Target};
use serde::Deserialize;
use std::path::Path;
use structopt::StructOpt;

mod csv_handler;
mod dto;
mod github;
mod store;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "ghworker",
    about = "A worker to pull data from github for our users"
)]
struct GithubWorker {
    #[structopt(short, long, env = "GITHUB_API_TOKEN")]
    token: String,
    #[structopt(short, long)]
    username: String,
    #[structopt(long, env = "STORE_TYPE")]
    store: String,
    #[structopt(long, env = "OUTPUT_LOCATION")]
    output: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    github_api_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = Builder::from_default_env();

    dotenv().ok();
    builder.target(Target::Stdout);
    builder.init();
    // let config: Config = envy::from_env().context("Failed to read the environment variables")?;

    let app = GithubWorker::from_args();
    if !(app.store == String::from("file") || app.store == String::from("database")) {
        return Err(anyhow!(
            "You can only provide a store of either 'file' or 'database'"
        ));
    }

    let pr_contributions =
        github::pull_request::get_user_pull_requests(app.token.clone(), app.username.clone())
            .await?;

    let contributions =
        github::contributions::get_user_contributions(app.token, app.username).await?;
    let contributions_file_path: String;
    match Path::new(&app.output).join("contributions.csv").to_str() {
        Some(path) => contributions_file_path = String::from(path),
        None => return Err(anyhow!("Could not parse the contributions file path")),
    };

    let store_type = match app.store.as_str() {
        "file" => StoreDestination::File(contributions_file_path),
        "database" => StoreDestination::Database,
        _ => StoreDestination::File(contributions_file_path),
    };

    let store = Store::new(store_type);
    store.store_contributions(&contributions);

    Ok(())
}
