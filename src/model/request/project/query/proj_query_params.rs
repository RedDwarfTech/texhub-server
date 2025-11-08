#[derive(serde::Deserialize)]
pub struct ProjQueryParams {
    pub tag: Option<String>,
    pub role_id: Option<i32>,
    pub proj_source_type: Option<i16>,
    #[serde(default = "default_archive_status")]
    pub archive_status: i32,
    #[serde(default = "default_trash")]
    pub trash: i32,
    #[serde(default = "default_proj_status")]
    pub proj_status: i32
}

fn default_archive_status() -> i32 {
    0
}

fn default_trash() -> i32 {
    0
}

/**
 * 0:待生成 1:生成中 2:已生成
 */
fn default_proj_status() -> i32 {
    1
}