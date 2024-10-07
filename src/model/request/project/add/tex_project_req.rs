use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexProjectReq {
    #[validate(length(max = 256))]
    pub name: String,
    pub template_id: Option<i64>,
    pub folder_id: Option<i64>,
    pub legacy_proj_id: Option<String>,
    pub proj_source_type: Option<i16>,
    pub proj_source: Option<String>
}