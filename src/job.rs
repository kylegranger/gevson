use std::{path::PathBuf, time::SystemTime};

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
    pub proof_path: String,
    pub schema: ProverSchema,
    pub timeout: u64,
    pub witness_url: String,
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
    pub data_directory: PathBuf,
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

    pub fn do_active(&mut self) {
        println!("job: do_active");
        if self.timed_out() {
            println!("  job timed out");
            self.state = JobState::TimedOut;
        }
    }

    pub fn do_pending(&mut self) {
        println!("job: do_pending");

        //
        // magic happens here!
        //

        println!("  set to active");
        self.state = JobState::Active;
    }
}
