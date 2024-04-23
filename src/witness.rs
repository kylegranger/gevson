use crate::types::{ProofRequest, ProverInput, ProverSource};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub struct Witness {
    inputs: Vec<ProverInput>,
    hash: Option<String>,
    url: Option<String>,
}

impl Witness {
    pub fn new(inputs: Vec<ProverInput>) -> Self {
        Self {
            inputs,
            hash: None,
            url: None,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        // match self.inputs.source {
        //     InputsSource::Url(_url) => {
        //         todo!()
        //     }
        //     InputsSource::Data(data) => {
        //         self.data = data.as_bytes().to_vec();
        //     }
        //     InputsSource::File(filepath) => {
        //         let filepath = PathBuf::from(filepath);
        //         let s = fs::read_to_string(filepath)?;
        //         tracing::info!("string from file length {}", s.len());
        //         self.data = s.as_bytes().to_vec();
        //     }
        // }
        // tracing::info!("witness data length {}", self.data.len());
        Ok(())
    }
}
