use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "step")]
pub enum Step {
    #[serde(rename = "smpp.send")]
    Send { credential: String },
}

pub(crate) const STEPS: &[&str] = &["smpp.send"];
