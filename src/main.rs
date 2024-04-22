mod job;

use clap::Parser;
use job::*;
use std::{
    env,
    path::PathBuf,
    thread::{self, sleep},
    time::{Duration, SystemTime},
};
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};

#[derive(Debug)]
struct GevsonEnv {
    upload_cmd: Option<String>,
    upload_url: Option<String>,
}

#[derive(Parser, Debug)]
#[clap(author = "Taiko Prover", version, about, long_about = None)]
pub struct ArgConfiguration {
    /// File path of witness file
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

fn get_env() -> GevsonEnv {
    let upload_cmd = match env::var("GEV_UPLOAD_CMD") {
        Ok(res) => Some(res),
        _ => None,
    };
    let upload_url = match env::var("GEV_UPLOAD_URL") {
        Ok(res) => Some(res),
        _ => None,
    };
    GevsonEnv {
        upload_cmd,
        upload_url,
    }
}

fn parse_args() -> (ProofRequest, PathBuf) {
    let args: Vec<_> = std::env::args().collect();
    let arg_conf = ArgConfiguration::parse_from(&args);

    // let filename = Path::new(&arg_conf.witness)
    //     .file_name()
    //     .unwrap()
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    let witness_path = PathBuf::from(&arg_conf.witness);
    let filename = witness_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let json_url = arg_conf
        .jsonurl
        .unwrap_or("http://localhost:9944".to_string());
    let proof_path = PathBuf::from(arg_conf.proof.unwrap_or("proof.json".to_string()));
    let timeout = arg_conf.timeout.unwrap_or(600);
    let schema = arg_conf.schema;
    let data_directory = PathBuf::from(arg_conf.datadir.unwrap_or("./".to_string()));

    (
        ProofRequest {
            filename,
            witness_path,
            json_url,
            proof_path,
            timeout,
            schema,
        },
        data_directory,
    )
}

fn start_logger(default_level: LevelFilter) {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        _ => EnvFilter::default().add_directive(default_level.into()),
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .init();

    // Comment above & uncomment below for tokio-console.
    //console_subscriber::init();
}

fn run_loop(jobs: &mut Vec<Job>) {
    loop {
        tracing::trace!("loop top");
        for job in &mut *jobs {
            match job.state {
                JobState::Pending => job.do_pending(),
                JobState::Active => job.do_active(),
                _ => (),
            }
        }
        let mut n = 0;
        for job in &mut *jobs {
            if job.state == JobState::Complete
                || job.state == JobState::Invalid
                || job.state == JobState::TimedOut
            {
                tracing::info!("removing job");
                jobs.remove(n);
                break;
            }
            n += 1;
        }
        if jobs.len() > 0 {
            sleep(Duration::from_millis(1000));
        } else {
            tracing::info!("done loop");
            break;
        }
    }
}

fn main() {
    start_logger(LevelFilter::INFO);
    let (proof_request, data_directory) = parse_args();
    let gevson_env = get_env();
    tracing::info!("proof_request: {:?}", proof_request);
    tracing::info!("gevson_env: {:?}", gevson_env);

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let mut jobs: Vec<Job> = Vec::new();
    jobs.push(Job {
        proof_request,
        data_directory,
        timestamp,
        txhash: None,
        state: JobState::Pending,
    });

    let work_thread = thread::spawn(move || {
        run_loop(&mut jobs);
    });

    let _result = work_thread.join().unwrap();
}
