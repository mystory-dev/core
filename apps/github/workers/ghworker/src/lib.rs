use anyhow::*;
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
    #[structopt(long, env = "DATABASE_URL")]
    pub database: String,
    #[structopt(long, env = "QUEUE_URL")]
    pub queue_url: String,
    #[structopt(long, env = "QUEUE_TOPIC")]
    pub queue_topic: String,
    #[structopt(long, env = "QUEUE_GROUP")]
    pub queue_group: String,
}

pub async fn run(db_pool: &PgPool, username: String, token: String) -> Result<()> {
    let mut worker = Worker::new(db_pool);

    worker
        .fetch_data_from_github(username, token)
        .await?
        .store_data()
        .await?;

    Ok(())
}
