use std::collections::HashMap;

#[derive(Debug)]
pub struct Review {
    pub id: String,
    pub occurred_at: String,
    pub viewer_did_author: bool,
    pub state: String,
    pub author_id: String,
}

#[derive(Debug)]
pub struct Commit {
    pub id: String,
    pub hash: String,
    pub author_id: String,
    pub occurred_at: String,
}

#[derive(Debug)]
pub struct PullRequest {
    pub id: String,
    pub author_id: String,
    pub date_opened: String,
    pub date_closed: Option<String>,
    pub number: i64,
    pub reviews: Vec<Review>,
    pub commits: Vec<Commit>,
}

#[derive(Debug)]
pub struct PullRequestsDTO {
    pub pull_requests: HashMap<String, PullRequest>,
}

impl PullRequestsDTO {
    pub fn new() -> PullRequestsDTO {
        PullRequestsDTO {
            pull_requests: HashMap::new(),
        }
    }

    pub fn add_pull_request(
        &mut self,
        id: String,
        date_opened: String,
        date_closed: Option<String>,
        author_id: String,
        number: i64,
    ) {
        self.pull_requests.insert(
            id.clone(),
            PullRequest {
                id,
                number,
                author_id,
                date_opened,
                date_closed,
                reviews: Vec::new(),
                commits: Vec::new(),
            },
        );
    }

    pub fn add_commit(
        &mut self,
        pull_request_id: &String,
        commit_id: String,
        occurred_at: String,
        hash: String,
        author_id: String,
    ) {
        match self.pull_requests.get_mut(pull_request_id) {
            Some(pull_request) => {
                pull_request.commits.push(Commit {
                    id: commit_id,
                    occurred_at,
                    hash,
                    author_id,
                });
            }
            _ => {}
        }
    }

    pub fn add_review(
        &mut self,
        pull_request_id: &String,
        review_id: String,
        occurred_at: String,
        viewer_did_author: bool,
        state: String,
        author_id: String,
    ) {
        match self.pull_requests.get_mut(pull_request_id) {
            Some(pull_request) => {
                pull_request.reviews.push(Review {
                    id: review_id,
                    occurred_at,
                    viewer_did_author,
                    state,
                    author_id,
                });
            }
            _ => {}
        }
    }
}
