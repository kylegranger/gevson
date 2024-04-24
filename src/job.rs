use crate::gevson::GevsonEnv;
use crate::types::{DataSource, ProofRequest};
use crate::witness::Witness;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use std::{path::Path, time::SystemTime};

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
pub struct Job {
    pub proof_request: ProofRequest,
    pub timestamp: u64,
    pub state: JobState,
    pub client_id: u64,
}

#[allow(dead_code)]
pub fn system_command(cmd: String) -> Result<()> {
    // This is a truly sucky thing about Rust. There, I said it.
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
    fn is_timed_out(&mut self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.timestamp + self.proof_request.timeout * 1000 < now
    }

    pub fn do_active(&mut self) -> Result<()> {
        if self.is_timed_out() {
            tracing::info!("job timed out");
            self.state = JobState::TimedOut;
        }
        Ok(())
    }
    #[allow(dead_code)]
    fn upload_file(&mut self, localfile: &PathBuf, gevson_env: &GevsonEnv) -> Result<String> {
        if gevson_env.upload_cmd.is_none() {
            tracing::warn!("No upload command template string");
            return Err(anyhow!("No upload command template string"));
        }
        if gevson_env.upload_url.is_none() {
            tracing::warn!("No upload url template string");
            return Err(anyhow!("No upload url template string"));
        }
        let mut cmd = gevson_env.upload_cmd.as_ref().unwrap().clone();
        let mut url = gevson_env.upload_url.as_ref().unwrap().clone();
        cmd = cmd.replace("UPLOAD_PATH", localfile.to_str().unwrap());
        cmd = cmd.replace("UPLOAD_FILE", &self.proof_request.inputs[0].name);
        url = url.replace("UPLOAD_FILE", &self.proof_request.inputs[0].name);
        tracing::info!("new upload cmd: {}", cmd);
        tracing::info!("new upload url: {}", url);
        let result = system_command(cmd)?;
        tracing::info!("upload_file system command result: {:?}", result);

        Ok(url)
    }

    pub fn do_pending(&mut self, data_directory: &str, gevson_env: &GevsonEnv) -> Result<()> {
        tracing::info!("job: do_pending: {:?}", self);

        // create our witness
        let mut witness = Witness::new(self.proof_request.inputs.clone());

        // If Url, Blob, or Text source, create local file
        // otherwise, use file passed in
        let localpath = witness.init_local_file(data_directory)?;

        // get checksm/hash
        let hash = extract_hash_from_file_content(&localpath)?;
        tracing::info!("hash returned: {:?}", hash);

        // if source is not url, upload file
        let url = match witness.inputs[0].source.clone() {
            DataSource::Url(url) => Ok(url),
            _ => self.upload_file(&localpath, gevson_env),
        }?;

        tracing::info!("witness url: {}", url);
        tracing::info!("set job to active");
        self.state = JobState::Active;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn extract_hash_from_file_content(path: &Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    let fd = std::fs::File::open(path)?;
    hasher.update_reader(fd)?;
    let checksum = hasher.finalize().to_string();
    Ok((&checksum).into())
}
