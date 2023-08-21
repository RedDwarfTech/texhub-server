use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct LatestCompile {
    pub path: String,
    pub project_id: String,
}