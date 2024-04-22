use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Witness {
    pub filename: String,
    pub data: Vec<u8>,
    pub hash: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug)]
pub enum WitnessSource {
    Url(String),
    Filepath(String),
    Data(Vec<u8>),
}

impl Witness {
    pub fn new(filename: String) -> Self {
        Self {
            data: Vec::new(),
            filename,
            hash: None,
            url: None,
        }
    }

    pub fn init(&mut self, source: WitnessSource) -> Result<()> {
        match source {
            // WitnessSource::Url(url) => {
            //     todo!()
            // }
            WitnessSource::Data(data) => {
                self.data = data;
            }
            WitnessSource::Filepath(filepath) => {
                let filepath = PathBuf::from(filepath);
                let s = fs::read_to_string(filepath)?;
                tracing::info!("string from file length {}", s.len());
                self.data = s.as_bytes().to_vec();
            }
            _ => (),
        }
        tracing::info!("witness data length {}", self.data.len());
        Ok(())
    }
}
