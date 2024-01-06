#[derive(serde::Deserialize)]
pub struct ProjQueryParams {
    pub tag: Option<String>,
    pub role_id: Option<i32>,
    #[serde(default = "default_archive_status")]
    pub archive_status: i32,
    #[serde(default = "default_trash")]
    pub trash: i32,
    #[serde(default = "default_proj_type")]
    pub proj_type: i32
}

fn default_archive_status() -> i32 {
    0
}

fn default_trash() -> i32 {
    0
}

fn default_proj_type() -> i32 {
    1
}