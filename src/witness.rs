#[allow(unused_imports)]
use crate::types::{DataSource, Prover, ProverInput, ProverSchema};
use anyhow::Result;
use std::path::PathBuf;
use std::{fs, fs::File, io::copy};

#[allow(dead_code)]
pub struct Witness {
    pub inputs: Vec<ProverInput>,
    pub hash: Option<String>,
    pub url: Option<String>,
}

impl Witness {
    pub fn new(inputs: Vec<ProverInput>) -> Self {
        Self {
            inputs,
            hash: None,
            url: None,
        }
    }

    pub fn init_local_file(&mut self, data_directory: &str) -> Result<PathBuf> {
        let localfile = format!("{}/{}", data_directory, self.inputs[0].name);
        let localpath = PathBuf::from(&localfile);
        match &self.inputs[0].source {
            DataSource::Url(url) => {
                tracing::info!("download from url: {}", url);
                let mut resp = reqwest::blocking::get(url)?;
                tracing::info!("resp: {:?}", resp);
                tracing::info!("localpath: {:?}", localpath);
                let mut out = File::create(&localpath).expect("failed to create file");
                copy(&mut resp, &mut out).expect("failed to copy content");
                tracing::info!("done creating file");
                return Ok(localpath);
            }
            DataSource::Blob(data) => {
                fs::write(&localpath, data)?;
                return Ok(localpath);
            }
            DataSource::Text(text) => {
                fs::write(&localpath, text)?;
                return Ok(localpath);
            }
            DataSource::File(filepath) => {
                return Ok(PathBuf::from(filepath));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::job::extract_hash_from_file_content;
    use crate::types::{DataSource, ProofRequest, Prover, ProverInput, ProverSchema};
    use crate::witness::Witness;
    use std::fs;

    #[test]
    fn test_data_source_blob() {
        println!("test: test_data_source_blob");

        let mut bytes = Vec::new();

        // 43-byte array, 0..42
        for i in 0..43 {
            bytes.push(i as u8);
        }
        let proof_request = ProofRequest {
            inputs: vec![ProverInput {
                name: "test-1234".to_string(),
                source: DataSource::Blob(bytes),
            }],
            outputs: vec!["proof.json".to_string()],
            prover: Prover {
                schema: ProverSchema::Katla,
                prover_hash: "1234abcd".to_string(),
                verifier_hash: "1234abcd".to_string(),
            },
            timeout: 10,
        };

        let mut witness = Witness::new(proof_request.inputs.clone());
        let localpath = witness.init_local_file("./data").unwrap();
        let inbytes = std::fs::read_to_string(localpath.clone())
            .unwrap()
            .as_bytes()
            .to_vec();

        println!("test localpath: {:?}", localpath);
        println!("read {} bytes", inbytes.len());
        assert_eq!(
            localpath.clone().into_os_string().into_string().unwrap(),
            "./data/test-1234".to_string()
        );
        assert_eq!(inbytes.len(), 43);
        assert_eq!(inbytes[7], 7);
        assert_eq!(inbytes[42], 42);

        let _ = fs::remove_file(localpath);
    }

    #[test]
    fn test_data_source_file() {
        println!("test: test_data_source_file");
        let proof_request = ProofRequest {
            inputs: vec![ProverInput {
                name: "test-5678".to_string(),
                source: DataSource::File("./testdata/witness-441240.json".to_string()),
            }],
            outputs: vec!["proof.json".to_string()],
            prover: Prover {
                schema: ProverSchema::Katla,
                prover_hash: "5678abcd".to_string(),
                verifier_hash: "5678abcd".to_string(),
            },
            timeout: 10,
        };

        let mut witness = Witness::new(proof_request.inputs.clone());
        let localpath = witness.init_local_file("./data").unwrap();
        let inbytes = std::fs::read_to_string(localpath.clone())
            .unwrap()
            .as_bytes()
            .to_vec();

        println!("test localpath: {:?}", localpath);
        println!("read {} bytes", inbytes.len());
        assert_eq!(
            localpath.clone().into_os_string().into_string().unwrap(),
            "./testdata/witness-441240.json".to_string()
        );
        assert_eq!(inbytes.len(), 26588);
    }

    #[test]
    fn test_data_source_url() {
        println!("test: test_data_source_url");
        let proof_request = ProofRequest {
            inputs: vec![ProverInput {
                name: "test-2001".to_string(),
                source: DataSource::Url(
                    "https://gevulot-test.eu-central-1.linodeobjects.com/witness-28020.json"
                        .to_string(),
                ),
            }],
            outputs: vec!["proof.json".to_string()],
            prover: Prover {
                schema: ProverSchema::Katla,
                prover_hash: "5678abcd".to_string(),
                verifier_hash: "5678abcd".to_string(),
            },
            timeout: 10,
        };

        let mut witness = Witness::new(proof_request.inputs.clone());
        let localpath = witness.init_local_file("./data").unwrap();
        let inbytes = std::fs::read_to_string(localpath.clone())
            .unwrap()
            .as_bytes()
            .to_vec();

        println!("test localpath: {:?}", localpath);
        println!("read {} bytes", inbytes.len());
        assert_eq!(
            localpath.clone().into_os_string().into_string().unwrap(),
            "./data/test-2001".to_string()
        );
        assert_eq!(inbytes.len(), 198235);

        let hash = extract_hash_from_file_content(&localpath).unwrap();
        println!("hash returned: {:?}", hash);
        assert_eq!(
            hash,
            "a113174a743a016c0b55f429548b1b80bc2fbebf721f2ca3aaeec132f581c835"
        );
    }
}
