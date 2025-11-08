use actix_multipart::form::{MultipartForm, tempfile::TempFile};

#[derive(Debug, MultipartForm)]
pub struct FullProjUpload {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
}