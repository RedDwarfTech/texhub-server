use validator::Validate;

#[derive(serde::Deserialize, Validate, Clone, Debug)]
pub struct PdfPreviewReq {
    #[validate(length(min = 1))]
    pub proj_id: String,
}