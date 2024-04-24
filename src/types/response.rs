use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::SystemTime;

#[allow(dead_code)]
pub enum ResponseType {
    UnparsableRequest,
    TimedOut,
    Success,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    success: bool,
    message: Option<String>,
    tx_result: Option<String>,
    duration_in_ms: u64,
}

impl Response {
    fn get_duration(start_in_ms: u64) -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now - start_in_ms
    }

    pub fn new_as_json(response_type: ResponseType, start_in_ms: u64) -> String {
        let duration_in_ms = Response::get_duration(start_in_ms);
        let response = match response_type {
            ResponseType::UnparsableRequest => Self {
                success: false,
                message: Some("Could not parse message as ProofRequest".to_string()),
                tx_result: None,
                duration_in_ms,
            },
            ResponseType::TimedOut => Self {
                success: false,
                message: Some("The proof request timed out".to_string()),
                tx_result: None,
                duration_in_ms,
            },
            _ => panic!("Unhandled response type -- this is a bug!"),
        };
        json!(response).to_string()
    }

    #[allow(dead_code)]
    pub fn new_from_result_as_json(
        response_type: ResponseType,
        result: String,
        duration_in_ms: u64,
    ) -> String {
        let response = match response_type {
            ResponseType::Success => Self {
                success: true,
                message: None,
                tx_result: Some(result),
                duration_in_ms,
            },
            _ => panic!("Unhandled response type -- this is a bug!"),
        };
        json!(response).to_string()
    }
}
