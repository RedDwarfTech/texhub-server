use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexProjectReq {
    #[validate(length(max = 2))]
    pub doc_name: String,
}