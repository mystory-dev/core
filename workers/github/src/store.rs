use crate::database::repository::{CommitRepository, PullRequestRepository, ReviewRepository};
use crate::dto::pull_requests::PullRequestsDTO;
use anyhow::*;
use sqlx::PgPool;

pub struct Store {}

impl Store {
    pub async fn store_pull_requests(
        db_pool: &PgPool,
        pull_requests_dto: &PullRequestsDTO,
    ) -> Result<()> {
        for pull_request in pull_requests_dto.pull_requests.iter() {
            let _ = PullRequestRepository::create(db_pool, pull_request.1).await?;
        }

        Ok(())
    }

    pub async fn store_reviews(
        db_pool: &PgPool,
        pull_requests_dto: &PullRequestsDTO,
    ) -> Result<()> {
        for (_, pull_request) in pull_requests_dto.pull_requests.iter() {
            for review in &pull_request.reviews {
                ReviewRepository::create(db_pool, &pull_request, review).await?;
            }
        }

        Ok(())
    }

    pub async fn store_commits(
        db_pool: &PgPool,
        pull_requests_dto: &PullRequestsDTO,
    ) -> Result<()> {
        for (_, pull_request) in pull_requests_dto.pull_requests.iter() {
            for commit in &pull_request.commits {
                CommitRepository::create(db_pool, &pull_request, commit).await?;
            }
        }

        Ok(())
    }
}
