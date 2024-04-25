use crate::job::{Job, JobState};
use crate::types::{ProofRequest, Response, ResponseType};
use anyhow::Result;
use simple_websockets::{Event, Message, Responder};
use std::collections::HashMap;
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone)]
pub struct GevsonEnv {
    pub upload_cmd: Option<String>,
    pub upload_url: Option<String>,
}

pub struct GevsonMsg {
    pub msg: String,
    pub client_id: u64,
}

#[allow(dead_code)]
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

    pub fn parse_proof_request(msg: &str) -> Result<ProofRequest> {
        let proof_request: ProofRequest = serde_json::from_str(msg)?;
        Ok(proof_request)
    }

    fn handle_incoming_messages(&mut self) {
        if self.incoming.len() > 0 {
            tracing::info!("we have incoming messages!");
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            for gm in &self.incoming {
                let res = Gevson::parse_proof_request(&gm.msg);
                if res.is_ok() {
                    let proof_request = res.unwrap();
                    let job = Job {
                        proof_request,
                        timestamp,
                        state: JobState::Pending,
                        client_id: gm.client_id,
                    };
                    tracing::info!("add new job: {:?}", job);
                    self.jobs.push(job);
                } else {
                    let response =
                        Response::new_as_json(ResponseType::UnparsableRequest, timestamp);
                    let gevmsg = GevsonMsg {
                        msg: response,
                        client_id: gm.client_id,
                    };
                    self.outgoing.push(gevmsg);
                }
            }
            self.incoming.clear();
        }
    }

    fn handle_outgoing_messages(&mut self) {
        if self.outgoing.len() > 0 {
            tracing::info!("we have outgoing messages!");
            for gm in &self.outgoing {
                let responder = self.clients.get(&gm.client_id).unwrap();
                responder.send(Message::Text(gm.msg.clone()));
            }
            self.outgoing.clear();
        }
    }

    /// The main task that runs every 100 ms
    /// - checks incoming & outgoing messages
    /// - jobs
    ///   - check for timed out requests
    ///   - poll gevulot for proof completion (every 10 seconds, per request)
    fn loop_task(&mut self) {
        self.handle_incoming_messages();
        self.handle_outgoing_messages();

        for job in &mut *self.jobs {
            let res = match job.state {
                JobState::Pending => job.handle_pending(&self.data_directory, &self.gevson_env),
                JobState::Active => job.handle_active(),
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
                if job.state == JobState::TimedOut {
                    let response = Response::new_as_json(ResponseType::TimedOut, job.timestamp);
                    let gevmsg = GevsonMsg {
                        msg: response,
                        client_id: job.client_id,
                    };
                    self.outgoing.push(gevmsg);
                }
                self.jobs.remove(n);
                break;
            }
            n += 1;
        }
        sleep(Duration::from_millis(100));
    }

    /// This is the beating heart of gevson.
    /// One loop to rule them all
    /// In the end, the events are fairly sparse
    /// - ws events
    /// - everything else happens in loop_task
    pub fn run(&mut self) {
        let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
        loop {
            if !event_hub.is_empty() {
                match event_hub.poll_event() {
                    Event::Connect(client_id, responder) => {
                        tracing::info!("A client connected with id #{}", client_id);
                        self.clients.insert(client_id, responder);
                    }
                    Event::Disconnect(client_id) => {
                        tracing::info!("Client #{} disconnected.", client_id);
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
                    }
                }
            }
            self.loop_task();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gevson::Gevson;
    use crate::types::{DataSource, ProofRequest, Prover, ProverInput, ProverSchema};
    use serde_json::json;

    #[test]
    fn test_proof_request_parsing() {
        let proof_request = ProofRequest {
            inputs: vec![ProverInput {
                name: "test-1234".to_string(),
                source: DataSource::Text("text".to_string()),
            }],
            outputs: vec!["proof.json".to_string()],
            prover: Prover {
                schema: ProverSchema::Katla,
                prover_hash: "1234abcd".to_string(),
                verifier_hash: "1234abcd".to_string(),
            },
            timeout: 10,
        };
        let jrequest = json!(proof_request).to_string();
        let res = Gevson::parse_proof_request(&jrequest);
        assert!(res.is_ok());

        // should fail with no timeout field
        let jbad = jrequest.replace("timeout", "time");
        let res = Gevson::parse_proof_request(&jbad);
        assert!(!res.is_ok());
    }
}
