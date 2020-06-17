pub type Date = String;

type Contribution = (Date, i64);

#[derive(Debug)]
pub struct ContributionsDTO {
    pub contributions: Vec<Contribution>,
}

impl ContributionsDTO {
    pub fn new() -> ContributionsDTO {
        ContributionsDTO {
            contributions: Vec::new(),
        }
    }

    pub fn add_contribution(&mut self, contribution: Contribution) {
        self.contributions.push(contribution)
    }
}
