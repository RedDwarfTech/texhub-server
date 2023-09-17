use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};

#[derive(Debug, MultipartForm)]
pub struct ProjUploadFile {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
    pub project_id: Text<String>,
    pub parent: Text<String>
}