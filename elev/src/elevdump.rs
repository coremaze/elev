use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

use super::{ElevEntry, ElevEntryError};

#[derive(Debug, Error)]
pub enum ElevDumpError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid elevdump version: expected 'elevdump version 2', got '{0}'")]
    InvalidVersion(String),

    #[error("ElevEntry error: {0}")]
    ElevEntryError(#[from] ElevEntryError),
}

#[derive(Debug)]
pub struct ElevDump {
    pub entries: Vec<ElevEntry>,
}

impl ElevDump {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ElevDumpError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    pub fn from_str(s: &str) -> Result<Self, ElevDumpError> {
        let reader = BufReader::new(s.as_bytes());
        Self::from_reader(reader)
    }

    fn from_reader<R: BufRead>(reader: R) -> Result<Self, ElevDumpError> {
        let mut lines = reader.lines();

        // Check the version
        if let Some(first_line) = lines.next() {
            let first_line = first_line?;
            if first_line.trim() != "elevdump version 2" {
                return Err(ElevDumpError::InvalidVersion(first_line));
            }
        } else {
            return Err(ElevDumpError::InvalidVersion("".to_string()));
        }

        let mut entries = Vec::new();

        for line in lines {
            let line = line?;
            if !line.trim().is_empty() {
                let entry = ElevEntry::from_line(line)?;
                entries.push(entry);
            }
        }

        Ok(ElevDump { entries })
    }
}
