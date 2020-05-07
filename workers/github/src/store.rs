use crate::csv_handler::CSVHandler;
use crate::github::contributions::ContributionsDTO;
use log::warn;
use std::path::Path;

pub struct Store {
    destination: StoreDestination,
}

pub enum StoreDestination {
    File(String),
    Database,
}

impl Store {
    pub fn new(destination: StoreDestination) -> Store {
        Store { destination }
    }

    pub fn store_contributions(&self, contributions: &ContributionsDTO) -> anyhow::Result<()> {
        match &self.destination {
            StoreDestination::File(file) => {
                self.store_contributions_to_file(contributions, file)?;
            }
            StoreDestination::Database => {
                warn!("Storing contributions to the database has not yet been implemented")
            }
        }
        Ok(())
    }

    fn store_contributions_to_file(
        &self,
        contributions: &ContributionsDTO,
        path: &String,
    ) -> anyhow::Result<bool> {
        CSVHandler::new(Path::new(path))
            .check_for_file_path(true)?
            .store_contributions(contributions)?;
        Ok(true)
    }
}
