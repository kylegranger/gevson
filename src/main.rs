mod gevson;
mod job;
mod types;
mod witness;
// mod wsserver;

// use crate::types::{ProofRequest, Prover, ProverInput, ProverSchema, ProverSource};
// use crate::wsserver::start_ws_server;
use clap::Parser;
use gevson::{Gevson, GevsonEnv};
// use job::*;
// use serde_json::json;
use std::sync::{Arc, Mutex};
use std::{
    env,
    // fs::write,
    // path::PathBuf,
    // thread::{self, sleep},
    // time::{Duration, SystemTime},
};
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};
// use witness::WitnessSource;

#[derive(Parser, Debug)]
#[clap(author = "Taiko Prover", version, about, long_about = None)]
pub struct ArgConfiguration {
    /// RPC url of the Gevulot node [default: http://localhost:9944]
    #[clap(short, long, value_parser)]
    pub jsonurl: Option<String>,
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

fn parse_args() -> (String, String) {
    let args: Vec<_> = std::env::args().collect();
    let arg_conf = ArgConfiguration::parse_from(&args);

    let data_directory = arg_conf.datadir.unwrap_or("./".to_string());
    let json_url = arg_conf
        .jsonurl
        .unwrap_or("http://localhost:9944".to_string());

    (data_directory, json_url)
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

// fn run_loop(
//     // jobs: &mut Vec<Job>,
//     arequests: Arc<Mutex<Vec<
// ) {
//     let mut jobs: Vec<Job> = Vec::new();
//     loop {
//         tracing::trace!("loop top");
//         let mut requests = arequests.lock().unwrap();
//         if requests.len() > 0 {
//             for request in requests {
//                 let job = Job {
//                     proof_request: request.clone(),
//                     data_directory: data_directory.clone(),
//                     gevson_env: gevson_env.clone(),
//                     timestamp: SystemTime::now()
//                         .duration_since(SystemTime::UNIX_EPOCH)
//                         .unwrap()
//                         .as_millis() as u64,
//                     json_url: json_url.clone(),
//                     state: JobState::Pending,
//                 };
//                 tracing::info!("add new job: {:?}", job);
//                 jobs.push(job);
//             }
//             requests.clear();
//         }

//         for job in &mut *jobs {
//             let res = match job.state {
//                 JobState::Pending => job.do_pending(),
//                 JobState::Active => job.do_active(),
//                 _ => Ok(()),
//             };
//             if res.is_err() {
//                 job.state = JobState::Invalid;
//             }
//         }
//         let mut n = 0;
//         for job in &mut *jobs {
//             if job.state == JobState::Complete
//                 || job.state == JobState::Invalid
//                 || job.state == JobState::TimedOut
//             {
//                 tracing::info!("removing job");
//                 jobs.remove(n);
//                 break;
//             }
//             n += 1;
//         }
//         // if jobs.len() > 0 {
//         sleep(Duration::from_millis(1000));
//         // }
//         // else {
//         //     tracing::info!("done loop");
//         //     break;
//         // }
//     }
// }

// #[tokio::main]
fn main() {
    start_logger(LevelFilter::INFO);
    let (data_directory, json_url) = parse_args();
    let gevson_env = get_env();
    let mut gevson = Gevson::new(data_directory, json_url, gevson_env);
    tracing::info!("ahsdf");
    // let arc_gevson = Arc::new(Mutex::new(gevson));
    gevson.run();

    // let ajobs = Arc::new(Mutex::new(Vec::<Job>::new()));
    // let arequests = Arc::new(Mutex::new(Vec::<ProofRequest>::new()));
    // let requests = Arc::clone(&arequests);

    // let mut jobs: Vec<Job> = Vec::new();
    // jobs.push(Job {
    //     proof_request,
    //     data_directory,
    //     gevson_env,
    //     timestamp,
    //     json_url,
    //     // txhash: None,
    //     state: JobState::Pending,
    // });

    // let work_thread = thread::spawn(move || {
    //     run_loop(Arc::clone(&arequests), data_directory, json_url, gevson_env);
    // });

    // let requests = Arc::clone(&arequests);
    // let _res = start_ws_server(Arc::clone(&arequests)).await;
    // let _res = work_thread.join().unwrap();
}
