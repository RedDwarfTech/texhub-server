use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};

#[derive(Debug, MultipartForm)]
pub struct ProjFullUploadFile {
    #[multipart(rename = "file")]
    pub file: TempFile,
    pub project_id: Text<String>
}