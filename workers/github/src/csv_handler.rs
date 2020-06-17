use crate::dto::contributions::ContributionsDTO;
use anyhow::*;
use csv::Writer;
use std::fs::File;
use std::path::Path;

pub struct CSVHandler<'a> {
    file_path: &'a Path,
}

impl<'a> CSVHandler<'a> {
    pub fn new(path: &Path) -> CSVHandler {
        CSVHandler { file_path: path }
    }

    pub fn check_for_file_path(self, create: bool) -> anyhow::Result<CSVHandler<'a>> {
        if !Path::new(&self.file_path).exists() {
            if create {
                File::create(&self.file_path)?;
            } else {
                let error_message = format!(
                    "The path {:?} does not exist to save the results in",
                    &self.file_path
                );
                return Err(anyhow!(error_message));
            }
        }
        Ok(self)
    }

    pub fn store_contributions(&self, contributions: &ContributionsDTO) -> Result<bool> {
        let mut wtr = Writer::from_path(&self.file_path)?;

        // Write the file header
        wtr.write_record(&["Date", "Count"])?;

        // Write the file body
        for contribution in &contributions.contributions {
            wtr.serialize(contribution)?;
        }

        Ok(true)
    }
}
