use anyhow::*;
use async_std::task;
use dotenv::dotenv;
use env_logger::{Builder, Target};
use ghworker::GithubWorker;
use kafka::consumer::{Consumer, FetchOffset};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::process;
use structopt::StructOpt;

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    user_id: String,
    token: String,
    username: String,
}

fn main() -> Result<()> {
    let app = make_app()?;
    let db_pool: PgPool = task::block_on(connect_to_database(&app.database))?;

    if let Err(e) = task::block_on(connect_to_queue(&app, &db_pool)) {
        error!("Error: {}", e);
        process::exit(1);
    }

    Ok(())
}

async fn connect_to_queue(app: &GithubWorker, db_pool: &PgPool) -> Result<()> {
    debug!("Connecting to a kafka broker....");
    debug!("[Queue URL]: {}", app.queue_url);
    debug!("[Queue Topic]: {}", app.queue_topic);
    debug!("[Queue Consumer Group]: {}", app.queue_group);

    let mut consumer = Consumer::from_hosts(vec![app.queue_url.clone()])
        .with_topic(app.queue_topic.clone())
        .with_fallback_offset(FetchOffset::Earliest)
        .with_group(app.queue_group.clone())
        .create()
        .unwrap();

    loop {
        for ms in consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                let event: Event = serde_json::from_slice(&m.value[..]).unwrap();
                let key: String = std::str::from_utf8(m.key).unwrap().to_string();

                if key == String::from("REGISTER_PLUGIN:GITHUB") {
                    debug!("Received an event for the user => {}", &event.username);
                    ghworker::run(&db_pool, event.username, event.token).await?;
                }
            }
            consumer.consume_messageset(ms).unwrap();
        }

        consumer.commit_consumed().unwrap();
    }
}

fn make_app() -> Result<GithubWorker> {
    let mut builder = Builder::from_default_env();

    dotenv().ok();
    builder.target(Target::Stdout);
    builder.init();

    let app = GithubWorker::from_args();

    debug!("Starting up the application...");

    Ok(app)
}

async fn connect_to_database(database_url: &str) -> Result<PgPool> {
    Ok(PgPool::new(database_url).await?)
}
