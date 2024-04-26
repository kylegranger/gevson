mod gevson;
mod job;
mod types;
mod witness;

use clap::Parser;
use gevson::{Gevson, GevsonEnv};
use std::env;
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};

#[derive(Parser, Debug)]
#[clap(author = "Taiko Prover", version, about, long_about = None)]
pub struct ArgConfiguration {
    /// RPC url of the Gevulot node [default: http://localhost:9944]
    #[clap(short, long, value_parser)]
    pub jsonurl: Option<String>,
    /// Data directory to store downloaded files [default: ./data ]
    #[clap(short, long, value_parser)]
    pub datadir: Option<String>,
    /// Port for the WebSocket server [default: 8080 ]
    #[clap(short, long, value_parser)]
    pub port: Option<u16>,
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
    tracing::info!("upload cmd {:?}", upload_cmd);
    tracing::info!("upload url {:?}", upload_url);
    GevsonEnv {
        upload_cmd,
        upload_url,
    }
}

fn parse_args() -> (String, String, u16) {
    let args: Vec<_> = std::env::args().collect();
    let arg_conf = ArgConfiguration::parse_from(&args);

    let data_directory = arg_conf.datadir.unwrap_or("./data".to_string());
    let json_url = arg_conf
        .jsonurl
        .unwrap_or("http://localhost:9944".to_string());
    let port = arg_conf.port.unwrap_or(8080);

    (data_directory, json_url, port)
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

fn main() {
    start_logger(LevelFilter::INFO);
    let (data_directory, json_url, port) = parse_args();
    let gevson_env = get_env();
    let mut gevson = Gevson::new(data_directory, json_url, gevson_env);
    gevson.run(port);
}
