mod job;
mod witness;

use clap::Parser;
use job::*;
use std::{
    env,
    path::PathBuf,
    thread::{self, sleep},
    time::{Duration, SystemTime},
};
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};
use witness::WitnessSource;

#[derive(Parser, Debug)]
#[clap(author = "Taiko Prover", version, about, long_about = None)]
pub struct ArgConfiguration {
    /// Name of the witness file
    #[clap(short, long, value_parser)]
    pub name: String,
    /// Witness filepath (this, or url)
    #[clap(short, long, value_parser)]
    pub filepath: Option<String>,
    /// Witness url (this, or filepath)
    #[clap(short, long, value_parser)]
    pub url: Option<String>,
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
    tracing::trace!("upload cmd {:?}", upload_cmd);
    tracing::trace!("upload url {:?}", upload_url);
    GevsonEnv {
        upload_cmd,
        upload_url,
    }
}

fn parse_args() -> (ProofRequest, String) {
    let args: Vec<_> = std::env::args().collect();
    let arg_conf = ArgConfiguration::parse_from(&args);
    let witness_name = arg_conf.name;

    // let witness_path = arg_conf.filepath;
    let witness_url = arg_conf.url.unwrap();

    // let filename = witness_path
    //     .file_name()
    //     .unwrap()
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    let json_url = arg_conf
        .jsonurl
        .unwrap_or("http://localhost:9944".to_string());
    let proof_path = PathBuf::from(arg_conf.proof.unwrap_or("proof.json".to_string()));
    let timeout = arg_conf.timeout.unwrap_or(600);
    let schema = arg_conf.schema;
    let data_directory = arg_conf.datadir.unwrap_or("./".to_string());

    (
        ProofRequest {
            witness_name,
            source: WitnessSource::Url(witness_url),
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
}

fn run_loop(jobs: &mut Vec<Job>) {
    loop {
        tracing::trace!("loop top");
        for job in &mut *jobs {
            let res = match job.state {
                JobState::Pending => job.do_pending(),
                JobState::Active => job.do_active(),
                _ => Ok(()),
            };
            if res.is_err() {
                job.state = JobState::Invalid;
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
        gevson_env,
        timestamp,
        txhash: None,
        state: JobState::Pending,
    });

    let work_thread = thread::spawn(move || {
        run_loop(&mut jobs);
    });

    let _result = work_thread.join().unwrap();
}
