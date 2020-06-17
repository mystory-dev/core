use crate::dto::pull_requests::{Commit, PullRequest, Review};
use anyhow::*;
use chrono::offset::FixedOffset;
use chrono::DateTime;
use log::debug;
use sqlx::PgPool;

pub struct CommitRepository {}

impl CommitRepository {
    pub async fn create(
        db_pool: &PgPool,
        pull_request: &PullRequest,
        commit: &Commit,
    ) -> Result<()> {
        debug!("About to save the commit => {}", &commit.id);

        let mut tx = db_pool.begin().await?;

        let modified_records = sqlx::query(
            r#"
            INSERT INTO commits (id, hash, date_committed, author_id, pull_request_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(&commit.id)
        .bind(&commit.hash)
        .bind(DateTime::parse_from_rfc3339(&commit.occurred_at)?)
        .bind(&commit.author_id)
        .bind(&pull_request.id)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        if modified_records > 0 {
            debug!("Commit {} added to the datbase", &commit.id);
        } else {
            debug!("Commit {} already existed", &commit.id);
        }

        Ok(())
    }
}

pub struct ReviewRepository {}

impl ReviewRepository {
    pub async fn create(
        db_pool: &PgPool,
        pull_request: &PullRequest,
        review: &Review,
    ) -> Result<()> {
        debug!("About to save the review => {}", &review.id);

        let mut tx = db_pool.begin().await?;

        let modified_records = sqlx::query(
            r#"
            INSERT INTO reviews (id, date_published, is_owner, state, author_id, pull_request_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(&review.id)
        .bind(DateTime::parse_from_rfc3339(&review.occurred_at)?)
        .bind(&review.viewer_did_author)
        .bind(&review.state)
        .bind(&review.author_id)
        .bind(&pull_request.id)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        if modified_records > 0 {
            debug!("Review {} added to the datbase", &review.id);
        } else {
            debug!("Review {} already existed", &review.id);
        }

        Ok(())
    }
}

pub struct PullRequestRepository {}

impl PullRequestRepository {
    pub async fn create(db_pool: &PgPool, pull_request: &PullRequest) -> Result<()> {
        debug!("About to save the pull request => {}", &pull_request.id);

        let mut tx = db_pool.begin().await?;
        let mut date_closed = None::<DateTime<FixedOffset>>;

        if let Some(date) = &pull_request.date_closed {
            date_closed = Some(DateTime::parse_from_rfc3339(date)?);
        }

        let modified_record = sqlx::query(
            r#"
            INSERT INTO pull_requests (id, author_id, date_opened, date_closed)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(&pull_request.id)
        .bind(&pull_request.author_id)
        .bind(DateTime::parse_from_rfc3339(&pull_request.date_opened)?)
        .bind(date_closed)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        debug!("Pull request {} added to the datbase", &modified_record);

        Ok(())
    }
}
