use crate::types::{DataSource, ProverInput};
use anyhow::Result;
use std::path::PathBuf;
use std::{fs, fs::File, io::copy};

#[allow(dead_code)]
pub struct Witness {
    pub inputs: Vec<ProverInput>,
    pub hash: Option<String>,
    pub url: Option<String>,
}

impl Witness {
    pub fn new(inputs: Vec<ProverInput>) -> Self {
        Self {
            inputs,
            hash: None,
            url: None,
        }
    }

    pub fn init_local_file(&mut self, data_directory: &str) -> Result<PathBuf> {
        let localfile = format!("{}/{}", data_directory, self.inputs[0].name);
        let localpath = PathBuf::from(&localfile);
        match &self.inputs[0].source {
            DataSource::Url(url) => {
                tracing::info!("download from url: {}", url);
                let mut resp = reqwest::blocking::get(url)?;
                tracing::info!("resp: {:?}", resp);
                tracing::info!("localpath: {:?}", localpath);
                let mut out = File::create(&localpath).expect("failed to create file");
                copy(&mut resp, &mut out).expect("failed to copy content");
                tracing::info!("done creating file");
                return Ok(localpath);
            }
            DataSource::Blob(data) => {
                fs::write(&localpath, data)?;
                return Ok(localpath);
            }
            DataSource::Text(text) => {
                fs::write(&localpath, text)?;
                return Ok(localpath);
            }
            DataSource::File(filepath) => {
                return Ok(PathBuf::from(filepath));
            }
        }
    }
}
