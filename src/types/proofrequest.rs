use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProverSource {
    Url(String),
    File(String),
    Text(String),
    Data(Vec<u8>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProverInput {
    pub name: String,
    pub source: ProverSource,
}

#[derive(PartialEq, Clone, Copy, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Prover {
    pub schema: ProverSchema,
    pub prover_hash: String,
    pub verifier_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProofRequest {
    pub inputs: Vec<ProverInput>,
    pub outputs: Vec<String>,
    pub prover: Prover,
    pub timeout: u64,
}
