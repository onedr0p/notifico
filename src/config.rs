use notifico_core::credentials::{Credential, Credentials};
use notifico_core::error::EngineError;
use notifico_core::pipeline::Pipeline;
use notifico_core::recipient::Recipient;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub project: String,
    pub pipelines: Vec<Pipeline>,
    pub credentials: Vec<Credential>,
    pub recipients: Vec<Recipient>,
}

pub struct SimpleCredentials {
    creds: Vec<Credential>,
}

impl SimpleCredentials {
    pub fn new(creds: Vec<Credential>) -> Self {
        SimpleCredentials { creds }
    }
}

impl Credentials for SimpleCredentials {
    fn get_credential(&self, r#type: &str, name: &str) -> Result<Value, EngineError> {
        for cred in self.creds.iter() {
            if cred.r#type == r#type && cred.name == name {
                return Ok(cred.value.clone());
            }
        }
        Err(EngineError::CredentialNotFound(
            r#type.to_string().into(),
            name.into(),
        ))
    }
}
