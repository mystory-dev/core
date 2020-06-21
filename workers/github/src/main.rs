use anyhow::*;
use async_std::task;
use dotenv::dotenv;
use env_logger::{Builder, Target};
use ghworker::GithubWorker;
use log::{debug, error};
use sqlx::PgPool;
use std::process;
use structopt::StructOpt;

fn main() -> Result<()> {
    let app = make_app()?;
    let db_pool: PgPool = task::block_on(connect_to_database(&app.database))?;

    if let Err(e) = task::block_on(ghworker::run(&app, &db_pool)) {
        error!("Error: {}", e);
        process::exit(1);
    }

    Ok(())
}

fn make_app() -> Result<GithubWorker> {
    let mut builder = Builder::from_default_env();

    dotenv().ok();
    builder.target(Target::Stdout);
    builder.init();

    let app = GithubWorker::from_args();
    if !(app.store == String::from("file") || app.store == String::from("database")) {
        return Err(anyhow!(
            "You can only provide a store of either 'file' or 'database'"
        ));
    }

    debug!("Aplication configuration:");
    debug!("Token => {}", &app.token);
    debug!("Username => {}\n", &app.username);

    Ok(app)
}

async fn connect_to_database(database_url: &str) -> Result<PgPool> {
    Ok(PgPool::new(database_url).await?)
}
