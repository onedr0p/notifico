use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "step")]
pub enum Step {
    #[serde(rename = "smtp.send")]
    Send { credential: String },
}

pub(crate) const STEPS: &[&str] = &["smtp.send"];
