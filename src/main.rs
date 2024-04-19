use clap::Parser;

#[derive(PartialEq, Clone, Debug, Copy)]
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

#[derive(Parser, Debug)]
#[clap(author = "Taiko Prover", version, about, long_about = None)]
pub struct ArgConfiguration {
    /// Points to public url of witness file
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
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ProofRequest {
    witness_url: String,
    json_url: String,
    proof_path: String,
    timeout: u64,
    schema: ProverSchema,
}

fn parse_args() -> ProofRequest {
    let args: Vec<_> = std::env::args().collect();
    let arg_conf = ArgConfiguration::parse_from(&args);

    let witness_url = arg_conf.witness;
    let json_url = arg_conf
        .jsonurl
        .unwrap_or("http://localhost:9944".to_string());
    let proof_path = arg_conf.proof.unwrap_or("proof.json".to_string());
    let timeout = arg_conf.timeout.unwrap_or(600);
    let schema = arg_conf.schema;

    ProofRequest {
        witness_url,
        json_url,
        proof_path,
        timeout,
        schema,
    }
}

fn main() {
    // let args: Vec<_> = std::env::args().collect();
    // let arg_conf = ArgConfiguration::parse_from(&args);

    let proof_request = parse_args();
    println!("proof_request: {:?}", proof_request);
    // let witness_url = arg_conf.witness;
    // let json_url = arg_conf
    //     .jsonurl
    //     .unwrap_or("http://localhost:9944".to_string());
    // let proof_path = arg_conf.proof.unwrap_or("proof.json".to_string());
    // let timeout = arg_conf.timeout.unwrap_or(600);
    // let schema = arg_conf.schema;

    // println!("witness_url: {:?}", witness_url);
    // println!("json_url: {:?}", json_url);
    // println!("proof_path: {:?}", proof_path);
    // println!("timeout: {:?}", timeout);
    // println!("witness_path: {:?}", witness_path);
}
