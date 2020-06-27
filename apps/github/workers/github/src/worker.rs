use crate::dto::PullRequestsDTO;
use crate::github::{get_pull_request_contributions, get_pull_request_review_contributions};
use crate::store::Store;
use anyhow::*;
use log::debug;
use sqlx::PgPool;

pub struct Worker<'a> {
    db_pool: &'a PgPool,
    pull_requests: Option<PullRequestsDTO>,
}

impl<'a> Worker<'a> {
    pub fn new(db_pool: &PgPool) -> Worker {
        Worker {
            db_pool,
            pull_requests: None::<PullRequestsDTO>,
        }
    }

    pub async fn fetch_data_from_github(
        &'a mut self,
        username: String,
        token: String,
    ) -> Result<&Worker<'a>> {
        let mut pr_contributions = PullRequestsDTO::new();
        get_pull_request_contributions(token.clone(), username.clone(), &mut pr_contributions)
            .await?;
        get_pull_request_review_contributions(
            token.clone(),
            username.clone(),
            &mut pr_contributions,
        )
        .await?;
        debug!(
            "Fetched {} pull request contributions for {}",
            pr_contributions.pull_requests.keys().len(),
            username
        );

        self.pull_requests = Some(pr_contributions);

        Ok(self)
    }

    pub async fn store_data(&'a self) -> Result<&Worker<'a>> {
        if let Some(pull_requests) = &self.pull_requests {
            Store::store_pull_requests(self.db_pool, pull_requests).await?;
            Store::store_reviews(self.db_pool, pull_requests).await?;
            Store::store_commits(self.db_pool, pull_requests).await?;
        }

        Ok(self)
    }
}
