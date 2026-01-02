use crate::{model::{diesel::custom::project::upload::proj_pdf_upload_file::ProjPdfUploadFile, request::project::query::download_proj::DownloadProj}, service::project::project_service::handle_compress_proj};
use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, MultipartFormConfig};
use crate::model::diesel::custom::project::upload::proj_upload_file::ProjUploadFile;
use crate::service::project::project_service::save_proj_output;
use actix_web::{HttpRequest, web, HttpResponse};
use mime::Mime;

pub async fn download_project(
    req: HttpRequest,
    form: web::Json<DownloadProj>,
) -> actix_web::Result<impl actix_web::Responder> {
    let path = handle_compress_proj(&form.0);
    match NamedFile::open(&path) {
        Ok(file) => {
            let content_type: Mime = "application/zip".parse().unwrap();
            Ok(NamedFile::set_content_type(file, content_type).into_response(&req))
        }
        Err(_) => Err(actix_web::error::ErrorBadRequest("File not Found")),
    }
}

async fn upload_project_output(
    MultipartForm(form): MultipartForm<ProjPdfUploadFile>,
) -> HttpResponse {
    save_proj_output(form).await
}

 pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/inner-tex/project")
        .route("/download", web::put().to(download_project))
        .route("/upload-output", web::post().to(upload_project_output))
    );
    // configure multipart limits for this inner upload endpoint
    let inner_upload_config = MultipartFormConfig::default()
        .total_limit(104857600) // 100 MB
        .memory_limit(209715200);
    cfg.service(web::scope("/inner-tex/ul/proj").app_data(inner_upload_config));
}
