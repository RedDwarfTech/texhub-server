use actix_multipart::form::{MultipartForm, tempfile::TempFile};

#[derive(Debug, MultipartForm)]
pub struct ProjUploadFile {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
}