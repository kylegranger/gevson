mod hash;

use crate::witness::{Witness, WitnessSource};
use anyhow::{anyhow, Result};
use std::process::Command;
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Clone)]
pub struct GevsonEnv {
    pub upload_cmd: Option<String>,
    pub upload_url: Option<String>,
}

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
    pub gevson_env: GevsonEnv,
    pub timestamp: u64,
    pub txhash: Option<String>,
    pub state: JobState,
}

pub fn system_command(cmd: String) -> Result<()> {
    // this is a truly sucky thing about Rust
    let mut parts = cmd.split_whitespace();
    let mut args = Vec::new();
    loop {
        let n = parts.next();
        if n.is_none() {
            break;
        }
        args.push(n.unwrap().to_string());
    }
    tracing::info!("got this array: {:?}", args);
    let arg0 = args[0].clone();
    args.remove(0);

    let output = Command::new(arg0)
        .args(args)
        .output()
        .expect("failed to execute process");

    tracing::info!("system_command status: {}", output.status);
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stderr = format!("{}", String::from_utf8_lossy(&output.stderr));
    let success = output.status.success();
    tracing::info!("success: {}", success);
    let res = match success {
        true => Ok(()),
        false => Err(anyhow!(stderr)),
    };
    res
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
    fn upload_file(&mut self, localfile: &String) -> Result<String> {
        // let
        if self.gevson_env.upload_cmd.is_none() {
            tracing::warn!("No upload command template string");
            return Err(anyhow!("No upload command template string"));
        }
        if self.gevson_env.upload_url.is_none() {
            tracing::warn!("No upload url template string");
            return Err(anyhow!("No upload url template string"));
        }
        let mut cmd = self.gevson_env.upload_cmd.as_ref().unwrap().clone();
        let mut url = self.gevson_env.upload_url.as_ref().unwrap().clone();
        cmd = cmd.replace("UPLOAD_PATH", localfile);
        cmd = cmd.replace("UPLOAD_FILE", &self.proof_request.witness_name);
        url = url.replace("UPLOAD_FILE", &self.proof_request.witness_name);
        tracing::info!("new upload cmd: {}", cmd);
        tracing::info!("new upload url: {}", url);
        _ = system_command(cmd)?;

        Ok(url)
    }

    pub fn do_pending(&mut self) -> Result<()> {
        tracing::info!("job: do_pending: {:?}", self);

        // create our witness
        let mut witness = Witness::new(self.proof_request.witness_name.clone());
        witness.init(self.proof_request.source.clone())?;

        // write the file and get checksum
        let localfile = format!("{}/{}", self.data_directory, witness.filename);
        let localpath = Path::new(&localfile);

        // let localpath = Path::new(&localpath);
        fs::write(localpath, witness.data)?;
        let hash = extract_hash_from_file_content(Path::new(localpath))?;
        tracing::info!("hash returned: {:?}", hash);

        // if source is not url, upload file
        let url = match self.proof_request.source.clone() {
            WitnessSource::Url(url) => Ok(url),
            _ => self.upload_file(&localfile),
        }?;

        tracing::info!("final witness url: {}", url);
        tracing::info!("set job to active");
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
