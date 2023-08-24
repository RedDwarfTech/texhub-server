use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexProjectReq {
    #[validate(length(max = 2))]
    pub name: String,
    pub template_id: Option<i64>
}