use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};

#[derive(Debug, MultipartForm)]
pub struct ProjPdfUploadFile {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
    pub project_id: Text<String>
}