use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};

#[derive(Debug, MultipartForm)]
pub struct FullProjUpload {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
    pub parent: Text<String>
}