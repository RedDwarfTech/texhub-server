use validator::Validate;

#[derive(serde::Deserialize, Validate, Clone, Debug)]
pub struct PdfPreviewSign {
    #[validate(length(min = 1))]
    pub proj_id: String,
    pub signature: String,
    pub expire: i64,
    pub access_key: String,
}