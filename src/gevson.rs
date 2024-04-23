use crate::job::{Job, JobState};
use crate::types::ProofRequest;
// use crate::wsserver::{start_ws_server, GevsonMsg};
use std::sync::{Arc, Mutex};
use std::{
    thread::{self, sleep},
    time::{Duration, SystemTime},
};

use simple_websockets::{Event, Message, Responder};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GevsonEnv {
    pub upload_cmd: Option<String>,
    pub upload_url: Option<String>,
}

pub struct GevsonMsg {
    pub msg: String,
    pub client_id: u64,
}

pub struct Gevson {
    data_directory: String,
    json_url: String,
    gevson_env: GevsonEnv,
    jobs: Vec<Job>,
    pub messages: Vec<GevsonMsg>,
}

impl Gevson {
    pub fn new(data_directory: String, json_url: String, gevson_env: GevsonEnv) -> Self {
        Self {
            data_directory,
            json_url,
            gevson_env,
            jobs: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn run_loop(&mut self) {
        // let mut this = arc_gevson.lock().unwrap();
        loop {
            tracing::trace!("loop top");
            // let mut requests = arequests.lock().unwrap();
            if self.messages.len() > 0 {
                // for gm in this.messages {
                //     let proof_request: ProofRequest = serde_json::from_str(&gm.msg).unwrap();
                //     let job = Job {
                //         proof_request,
                //         // data_directory: sedata_directory.clone(),
                //         // gevson_env: gevson_env.clone(),
                //         timestamp: SystemTime::now()
                //             .duration_since(SystemTime::UNIX_EPOCH)
                //             .unwrap()
                //             .as_millis() as u64,
                //         // json_url: json_url.clone(),
                //         state: JobState::Pending,
                //     };
                //     tracing::info!("add new job: {:?}", job);
                //     this.jobs.push(job);
                // }
                self.messages.clear();
            }

            for job in &mut *self.jobs {
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
            for job in &mut *self.jobs {
                if job.state == JobState::Complete
                    || job.state == JobState::Invalid
                    || job.state == JobState::TimedOut
                {
                    tracing::info!("removing job");
                    self.jobs.remove(n);
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

    fn loop_task(&mut self) {
        // let mut this = arc_gevson.lock().unwrap();
        // loop {
        tracing::trace!("loop top");
        // let mut requests = arequests.lock().unwrap();
        if self.messages.len() > 0 {
            // for gm in this.messages {
            //     let proof_request: ProofRequest = serde_json::from_str(&gm.msg).unwrap();
            //     let job = Job {
            //         proof_request,
            //         // data_directory: sedata_directory.clone(),
            //         // gevson_env: gevson_env.clone(),
            //         timestamp: SystemTime::now()
            //             .duration_since(SystemTime::UNIX_EPOCH)
            //             .unwrap()
            //             .as_millis() as u64,
            //         // json_url: json_url.clone(),
            //         state: JobState::Pending,
            //     };
            //     tracing::info!("add new job: {:?}", job);
            //     this.jobs.push(job);
            // }
            self.messages.clear();
        }

        for job in &mut *self.jobs {
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
        for job in &mut *self.jobs {
            if job.state == JobState::Complete
                || job.state == JobState::Invalid
                || job.state == JobState::TimedOut
            {
                tracing::info!("removing job");
                self.jobs.remove(n);
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
        // }
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

    pub fn run(data_directory: String, json_url: String, gevson_env: GevsonEnv) {
        let mut gevson = Gevson::new(data_directory, json_url, gevson_env);

        // let this = arc_gevson.clone();
        tracing::info!("run lola run");
        // let arc_gevson = Arc::new(Mutex::new(gevson));
        // let work_thread = thread::spawn(move || {
        //     // let mut this = arc_gevson.lock().unwrap();
        //     Gevson::run_loop(&mut gevson);
        // });
        // let this = arc_gevson.clone();
        // let work_thread = thread::spawn(move || {
        //     Gevson::run_loop(this);
        // });

        // let _res = start_ws_server(this).await;

        // listen for WebSockets on port 8080:
        tracing::info!("start ws");
        let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
        // map between client ids and the client's `Responder`:
        let mut clients: HashMap<u64, Responder> = HashMap::new();
        tracing::info!("started ws seever");

        loop {
            tracing::info!("top loop");
            if !event_hub.is_empty() {
                tracing::info!("not empty");
                match event_hub.poll_event() {
                    Event::Connect(client_id, responder) => {
                        println!("A client connected with id #{}", client_id);
                        // add their Responder to our `clients` map:
                        clients.insert(client_id, responder);
                    }
                    Event::Disconnect(client_id) => {
                        println!("Client #{} disconnected.", client_id);
                        // remove the disconnected client from the clients map:
                        clients.remove(&client_id);
                    }
                    Event::Message(client_id, message) => {
                        println!(
                            "Received a message from client #{}: {:?}",
                            client_id, message
                        );
                        let msg: String = match message {
                            Message::Text(text) => text,
                            _ => "unhandled binary".to_string(),
                        };
                        let gevson_msg = GevsonMsg { msg, client_id };
                        gevson.messages.push(gevson_msg);
                        // retrieve this client's `Responder`:
                        // let responder = clients.get(&client_id).unwrap();
                        // // echo the message back:
                        // responder.send(message);
                    }
                }
            }
            gevson.loop_task();
        }
        // tracing::info!("done run");
    }
}
