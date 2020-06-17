use anyhow::*;
use async_std::task;
use dotenv::dotenv;
use env_logger::{Builder, Target};
use log::{debug, error};
use serde::Deserialize;
use sqlx::PgPool;
use store::Store;
use structopt::StructOpt;

mod csv_handler;
mod database;
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
    #[structopt(long, env = "DATABASE_URL")]
    database: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    github_api_token: String,
}

fn main() -> Result<()> {
    let app = make_app()?;

    let mut db_pool = task::block_on(connect_to_database(&app.database))?;

    let pr_contributions = task::block_on(github::pull_request::get_user_pull_requests(
        app.token.clone(),
        app.username.clone(),
    ))?;
    debug!(
        "Fetched {} pull request contributions for {}",
        pr_contributions.pull_requests.keys().len(),
        app.username
    );
    debug!("{:?}", pr_contributions.pull_requests.keys());

    let stored_pull_requests =
        task::block_on(Store::store_pull_requests(&db_pool, &pr_contributions))?;

    let stored_reviews = task::block_on(Store::store_reviews(&db_pool, &pr_contributions))?;
    let stored_commits = task::block_on(Store::store_commits(&db_pool, &pr_contributions))?;

    debug!("Bugging out!");

    Ok(())
}

fn make_app() -> Result<GithubWorker> {
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

    debug!("Aplication configuration:");
    debug!("Token => {}", &app.token);
    debug!("Username => {}", &app.username);
    debug!("\n");

    Ok(app)
}

async fn connect_to_database(database_url: &str) -> Result<PgPool> {
    Ok(PgPool::new(database_url).await?)
}
