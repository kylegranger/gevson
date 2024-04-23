use crate::job::{Job, JobState};
use crate::types::ProofRequest;
use crate::wsserver::{start_ws_server, GevsonMsg};
use std::sync::{Arc, Mutex};
use std::{
    env,
    fs::write,
    path::PathBuf,
    thread::{self, sleep},
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone)]
pub struct GevsonEnv {
    pub upload_cmd: Option<String>,
    pub upload_url: Option<String>,
}

pub struct Gevson<'a> {
    data_directory: String,
    json_url: String,
    gevson_env: GevsonEnv,
    jobs: Vec<Job>,
    messages: Vec<&'a GevsonMsg<'a>>,
}

impl<'a> Gevson<'a> {
    pub fn new(data_directory: String, json_url: String, gevson_env: GevsonEnv) -> Self {
        Self {
            data_directory,
            json_url,
            gevson_env,
            jobs: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn run_loop(arc_gevson: Arc<Mutex<Self>>) {
        let mut this = arc_gevson.lock().unwrap();
        loop {
            tracing::trace!("loop top");
            // let mut requests = arequests.lock().unwrap();
            if this.messages.len() > 0 {
                for gm in this.messages {
                    let proof_request: ProofRequest = serde_json::from_str(&gm.msg).unwrap();
                    let job = Job {
                        proof_request,
                        // data_directory: sedata_directory.clone(),
                        // gevson_env: gevson_env.clone(),
                        timestamp: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        // json_url: json_url.clone(),
                        state: JobState::Pending,
                    };
                    tracing::info!("add new job: {:?}", job);
                    this.jobs.push(job);
                }
                this.messages.clear();
            }

            for job in &mut *this.jobs {
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
            for job in &mut *this.jobs {
                if job.state == JobState::Complete
                    || job.state == JobState::Invalid
                    || job.state == JobState::TimedOut
                {
                    tracing::info!("removing job");
                    this.jobs.remove(n);
                    break;
                }
                n += 1;
            }
            // if jobs.len() > 0 {
            sleep(Duration::from_millis(1000));
            // }
            // else {
            //     tracing::info!("done loop");
            //     break;
            // }
        }
    }

    // pub async fn start_ws_server(
    //     arequests: Arc<Mutex<Vec<ProofRequest>>>,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let addr = "127.0.0.1:3000".to_string();

    //     // Start server
    //     let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    //     tracing::info!("Listening on: {}", addr);

    //     // Handle connections
    //     while let Ok((stream, _)) = listener.accept().await {
    //         let peer = stream
    //             .peer_addr()
    //             .expect("connected streams should have a peer address");
    //         tracing::info!("Peer address: {}", peer);

    //         tokio::spawn(accept_connection(peer, stream, Arc::clone(&arequests)));
    //     }

    //     Ok(())
    // }

    pub async fn run(arc_gevson: Arc<Mutex<Gevson<'a>>>) {
        let this = arc_gevson.clone();
        let work_thread = thread::spawn(move || {
            Gevson::run_loop(this);
        });
        let this = arc_gevson.clone();
        // let work_thread = thread::spawn(move || {
        //     Gevson::run_loop(this);
        // });

        let _res = start_ws_server(this).await;
    }
}
