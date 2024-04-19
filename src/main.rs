use clap::Parser;
use std::{
    path::PathBuf,
    thread,
    thread::sleep,
    time::{Duration, SystemTime},
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

#[derive(Parser, Debug)]
#[clap(author = "Taiko Prover", version, about, long_about = None)]
pub struct ArgConfiguration {
    /// Public url of witness file
    #[clap(short, long, value_parser)]
    pub witness: String,
    /// Timeout in seconds. Default is 600
    #[clap(short, long, value_parser)]
    pub timeout: Option<u64>,
    /// RPC url of the Gevulot node [default: http://localhost:9944]
    #[clap(short, long, value_parser)]
    pub jsonurl: Option<String>,
    /// Path of output proof [default: proof.json]
    #[clap(short, long, value_parser)]
    pub proof: Option<String>,
    /// katla | mock | polygon | sp1
    #[clap(short, long, value_parser)]
    pub schema: ProverSchema,
    /// Data directory to store downloaded files [default: ./ ]
    #[clap(short, long, value_parser)]
    pub datadir: Option<String>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ProofRequest {
    json_url: String,
    proof_path: String,
    schema: ProverSchema,
    timeout: u64,
    witness_url: String,
}

#[derive(PartialEq, Clone, Debug)]
#[allow(dead_code)]
enum JobState {
    Pending,
    Active,
    Complete,
    TimedOut,
    Invalid,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct GevsonJob {
    proof_request: ProofRequest,
    data_directory: PathBuf,
    timestamp: u64,
    txhash: Option<String>,
    state: JobState,
}

impl GevsonJob {
    fn timed_out(&mut self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.timestamp + self.proof_request.timeout * 1000 < now
    }
}

fn parse_args() -> (ProofRequest, PathBuf) {
    let args: Vec<_> = std::env::args().collect();
    let arg_conf = ArgConfiguration::parse_from(&args);

    let witness_url = arg_conf.witness;
    let json_url = arg_conf
        .jsonurl
        .unwrap_or("http://localhost:9944".to_string());
    let proof_path = arg_conf.proof.unwrap_or("proof.json".to_string());
    let timeout = arg_conf.timeout.unwrap_or(600);
    let schema = arg_conf.schema;
    let data_directory = PathBuf::from(arg_conf.datadir.unwrap_or("./".to_string()));

    (
        ProofRequest {
            witness_url,
            json_url,
            proof_path,
            timeout,
            schema,
        },
        data_directory,
    )
}

fn handle_pending_job(job: &mut GevsonJob) -> bool {
    println!("handle_pending_job");
    println!("  set to active");
    job.state = JobState::Active;
    let invalid = false;
    invalid
}

fn handle_active_job(job: &mut GevsonJob) -> bool {
    println!("handle_active_job");
    if job.timed_out() {
        println!("  job timed out");
        job.state = JobState::TimedOut;
        return true;
    }
    false
}

fn run_loop(jobs: &mut Vec<GevsonJob>) {
    loop {
        println!("\nloop top");
        let mut remove_jobs = false;
        if jobs.len() > 0 {
            for job in &mut *jobs {
                match job.state {
                    JobState::Pending => {
                        handle_pending_job(job);
                    }
                    JobState::Active => {
                        if handle_active_job(job) {
                            remove_jobs = true;
                        }
                    }
                    JobState::Complete => {}
                    JobState::TimedOut => {}
                    JobState::Invalid => {}
                }
            }
        } else {
            break;
        }
        if remove_jobs {
            let mut n = 0;
            for job in &mut *jobs {
                if job.state == JobState::Complete
                    || job.state == JobState::Invalid
                    || job.state == JobState::TimedOut
                {
                    println!("removing job");
                    jobs.remove(n);
                    break;
                }
                n += 1;
            }
        }
        sleep(Duration::from_millis(1000));
    }
    println!("done loop")
}

fn main() {
    let (proof_request, data_directory) = parse_args();
    println!("proof_request: {:?}", proof_request);

    let mut jobs: Vec<GevsonJob> = Vec::new();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    println!("timestamp: {:?}", timestamp);
    jobs.push(GevsonJob {
        proof_request,
        data_directory,
        timestamp,
        txhash: None,
        state: JobState::Pending,
    });

    let work_thread = thread::spawn(move || {
        // Some expensive computation.
        run_loop(&mut jobs);
    });

    let _result = work_thread.join().unwrap();
}
