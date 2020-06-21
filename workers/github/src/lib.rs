use anyhow::*;
use log::debug;
use sqlx::PgPool;
use structopt::StructOpt;
use worker::Worker;

mod database;
mod dto;
mod github;
mod store;
mod worker;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "ghworker",
    about = "A worker to pull data from github for our users"
)]
pub struct GithubWorker {
    #[structopt(short, long, env = "GITHUB_API_TOKEN")]
    pub token: String,
    #[structopt(short, long)]
    pub username: String,
    #[structopt(long, env = "STORE_TYPE")]
    pub store: String,
    #[structopt(long, env = "OUTPUT_LOCATION")]
    pub output: String,
    #[structopt(long, env = "DATABASE_URL")]
    pub database: String,
}

pub async fn run(app: &GithubWorker, db_pool: &PgPool) -> Result<()> {
    let mut worker = Worker::new(db_pool);

    worker
        .fetch_data_from_github(app.username.clone(), app.token.clone())
        .await?
        .store_data()
        .await?;

    debug!("Bugging out!");

    Ok(())
}
