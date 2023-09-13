use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexProjectTplReq {
    pub template_id: i64
}