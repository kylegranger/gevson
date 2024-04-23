use crate::job::{Job, JobState};
use crate::types::ProofRequest;
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Result};
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
    pub incoming: Vec<GevsonMsg>,
    pub outgoing: Vec<GevsonMsg>,
    clients: HashMap<u64, Responder>,
}

impl Gevson {
    pub fn new(data_directory: String, json_url: String, gevson_env: GevsonEnv) -> Self {
        Self {
            data_directory,
            json_url,
            gevson_env,
            jobs: Vec::new(),
            incoming: Vec::new(),
            outgoing: Vec::new(),
            clients: HashMap::new(),
        }
    }

    // fn run_loop(&mut self) {
    //     // let mut this = arc_gevson.lock().unwrap();
    //     loop {
    //         // tracing::trace!("loop top");
    //         // let mut requests = arequests.lock().unwrap();
    //         if self.messages.len() > 0 {
    //             // for gm in this.messages {
    //             //     let proof_request: ProofRequest = serde_json::from_str(&gm.msg).unwrap();
    //             //     let job = Job {
    //             //         proof_request,
    //             //         // data_directory: sedata_directory.clone(),
    //             //         // gevson_env: gevson_env.clone(),
    //             //         timestamp: SystemTime::now()
    //             //             .duration_since(SystemTime::UNIX_EPOCH)
    //             //             .unwrap()
    //             //             .as_millis() as u64,
    //             //         // json_url: json_url.clone(),
    //             //         state: JobState::Pending,
    //             //     };
    //             //     tracing::info!("add new job: {:?}", job);
    //             //     this.jobs.push(job);
    //             // }
    //             self.messages.clear();
    //         }

    //         for job in &mut *self.jobs {
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
    //         for job in &mut *self.jobs {
    //             if job.state == JobState::Complete
    //                 || job.state == JobState::Invalid
    //                 || job.state == JobState::TimedOut
    //             {
    //                 tracing::info!("removing job");
    //                 self.jobs.remove(n);
    //                 break;
    //             }
    //             n += 1;
    //         }
    //         // if jobs.len() > 0 {
    //         sleep(Duration::from_millis(100));
    //         // }
    //         // else {
    //         //     tracing::info!("done loop");
    //         //     break;
    //         // }
    //     }
    // }

    fn parse_proof_request(msg: &str) -> Result<ProofRequest> {
        let proof_request: ProofRequest = serde_json::from_str(msg)?;
        Ok(proof_request)
    }

    fn handle_incoming_messages(&mut self) {
        if self.incoming.len() > 0 {
            tracing::info!("we have incoming");
            for gm in &self.incoming {
                let res = Gevson::parse_proof_request(&gm.msg);
                if res.is_ok() {
                    let proof_request = res.unwrap();
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
                    self.jobs.push(job);
                } else {
                    let response = GevsonMsg {
                        msg: "Could not parse message as ProofRequest".to_string(),
                        client_id: gm.client_id,
                    };
                    self.outgoing.push(response);
                }
            }
            self.incoming.clear();
        }
    }

    fn handle_outgoing_messages(&mut self) {
        if self.outgoing.len() > 0 {
            tracing::info!("we have outgoing");
            for gm in &self.outgoing {
                let responder = self.clients.get(&gm.client_id).unwrap();
                responder.send(Message::Text(gm.msg.clone()));
            }
            self.outgoing.clear();
        }
    }
    fn loop_task(&mut self) {
        self.handle_incoming_messages();
        self.handle_outgoing_messages();

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
        sleep(Duration::from_millis(100));
    }

    pub fn run(&mut self) {
        let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
        loop {
            if !event_hub.is_empty() {
                match event_hub.poll_event() {
                    Event::Connect(client_id, responder) => {
                        tracing::info!("A client connected with id #{}", client_id);
                        // add their Responder to our `clients` map:
                        self.clients.insert(client_id, responder);
                    }
                    Event::Disconnect(client_id) => {
                        tracing::info!("Client #{} disconnected.", client_id);
                        // remove the disconnected client from the clients map:
                        self.clients.remove(&client_id);
                    }
                    Event::Message(client_id, message) => {
                        tracing::info!(
                            "Received a message from client #{}: {:?}",
                            client_id,
                            message
                        );
                        let msg: String = match message {
                            Message::Text(text) => text,
                            _ => "unhandled binary".to_string(),
                        };
                        let request = GevsonMsg { msg, client_id };
                        tracing::info!("adding new request");
                        self.incoming.push(request);
                        // retrieve this client's `Responder`:
                        // let responder = clients.get(&client_id).unwrap();
                        // // echo the message back:
                        // responder.send(message);
                    }
                }
            }
            self.loop_task();
        }
    }
}
