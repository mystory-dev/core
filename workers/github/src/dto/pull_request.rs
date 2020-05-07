use std::collections::HashMap;

#[derive(Debug)]
pub enum ReviewState {
    Pending,
    Commented,
    Approved,
    Changes,
    Dismissed,
}

#[derive(Debug)]
pub struct Review {
    id: String,
    occurred_at: String,
    viewer_did_author: bool,
    state: ReviewState,
}

#[derive(Debug)]
pub struct Commit {
    id: String,
    occurred_at: String,
}

#[derive(Debug)]
pub struct PullRequest {
    id: String,
    occurred_at: String,
    reviews: Vec<Review>,
    commits: Vec<Commit>,
}

#[derive(Debug)]
pub struct PullRequestsDTO {
    pull_requests: HashMap<String, PullRequest>,
}

impl PullRequestsDTO {
    pub fn new() -> PullRequestsDTO {
        PullRequestsDTO {
            pull_requests: HashMap::new(),
        }
    }

    pub fn add_pull_request(&mut self, id: String, occurred_at: String) {
        self.pull_requests.insert(
            id.clone(),
            PullRequest {
                id,
                occurred_at,
                reviews: Vec::new(),
                commits: Vec::new(),
            },
        );
    }

    pub fn add_commit(&mut self, pull_request_id: &String, commit_id: String, occurred_at: String) {
        match self.pull_requests.get_mut(pull_request_id) {
            Some(pull_request) => {
                pull_request.commits.push(Commit {
                    id: commit_id,
                    occurred_at,
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
        state: ReviewState,
    ) {
        match self.pull_requests.get_mut(pull_request_id) {
            Some(pull_request) => {
                pull_request.reviews.push(Review {
                    id: review_id,
                    occurred_at,
                    viewer_did_author,
                    state,
                });
            }
            _ => {}
        }
    }
}
