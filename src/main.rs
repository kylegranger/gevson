mod job;

use clap::Parser;
use job::*;
use std::{
    path::PathBuf,
    thread,
    thread::sleep,
    time::{Duration, SystemTime},
};

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

// fn handle_pending_job(job: &mut GevsonJob) -> bool {
//     job.do_pending()
//     // println!("handle_pending_job");
//     // println!("  set to active");
//     // job.state = JobState::Active;
//     // let invalid = false;
//     // invalid
// }

// fn handle_active_job(job: &mut GevsonJob) -> bool {
//     job.do_active()
//     // println!("handle_active_job");
//     // if job.timed_out() {
//     //     println!("  job timed out");
//     //     job.state = JobState::TimedOut;
//     //     return true;
//     // }
//     // false
// }

fn run_loop(jobs: &mut Vec<GevsonJob>) {
    loop {
        println!("\nloop top");
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
                println!("removing job");
                jobs.remove(n);
                break;
            }
            n += 1;
        }
        if jobs.len() > 0 {
            sleep(Duration::from_millis(1000));
        } else {
            println!("done loop");
            break;
        }
    }
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
        run_loop(&mut jobs);
    });

    let _result = work_thread.join().unwrap();
}
