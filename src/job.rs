mod hash;

use crate::witness::{Witness, WitnessSource};
use anyhow::Result;
// use hash::Hash;
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum ProverSchema {
    Katla,
    Mock,
    Polygon,
    Sp1,
}

impl From<&str> for ProverSchema {
    fn from(input: &str) -> ProverSchema {
        match input {
            "katla" => ProverSchema::Katla,
            "mock" => ProverSchema::Mock,
            "polygon" => ProverSchema::Polygon,
            "sp1" => ProverSchema::Sp1,
            _ => panic!("invalid mode string: {input}"),
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ProofRequest {
    pub json_url: String,
    pub witness_name: String,
    pub proof_path: PathBuf,
    pub schema: ProverSchema,
    pub timeout: u64,
    pub source: WitnessSource,
}

#[derive(PartialEq, Clone, Debug)]
#[allow(dead_code)]
pub enum JobState {
    Pending,
    Active,
    Complete,
    TimedOut,
    Invalid,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Job {
    pub proof_request: ProofRequest,
    pub data_directory: String,
    pub timestamp: u64,
    pub txhash: Option<String>,
    pub state: JobState,
}

impl Job {
    fn timed_out(&mut self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.timestamp + self.proof_request.timeout * 1000 < now
    }

    pub fn do_active(&mut self) -> Result<()> {
        tracing::trace!("job: do_active");
        if self.timed_out() {
            tracing::info!("  job timed out");
            self.state = JobState::TimedOut;
        }
        Ok(())
    }

    pub fn do_pending(&mut self) -> Result<()> {
        tracing::info!("job: do_pending: {:?}", self);

        // create our witness
        let mut witness = Witness::new(self.proof_request.witness_name.clone());
        witness.init(self.proof_request.source.clone())?;

        // write the file and get checksum
        let localpath = format!("{}/{}", self.data_directory, witness.filename);
        let localpath = Path::new(&localpath);
        fs::write(localpath, witness.data)?;
        // fs::write(
        //     Path::new("./proof.json"),
        //     serde_json::to_vec(&block_proof_data).unwrap(),
        // )
        // .unwrap();
        // tracing::trace!("job: witness_path: {:?}", self.proof_request.witness_path);
        let hash = extract_hash_from_file_content(localpath)?;
        tracing::info!("hash returned: {:?}", hash);
        tracing::info!("  set to active");
        self.state = JobState::Active;
        Ok(())
    }
}

pub fn extract_hash_from_file_content(path: &Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    let fd = std::fs::File::open(path)?;
    hasher.update_reader(fd)?;
    let checksum = hasher.finalize().to_string();
    Ok((&checksum).into())
}
