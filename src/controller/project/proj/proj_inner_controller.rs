use crate::{model::{diesel::custom::project::upload::proj_pdf_upload_file::ProjPdfUploadFile, request::project::query::download_proj::DownloadProj}, service::project::proj::project_service::{handle_compress_proj_async, save_full_proj_output}};
use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, MultipartFormConfig};
use log::info;
use crate::service::project::proj::project_service::save_proj_output;
use actix_web::{HttpRequest, web, HttpResponse};
use mime::Mime;
use crate::model::diesel::custom::project::upload::proj_full_upload_file::ProjFullUploadFile;

pub async fn download_project(
    req: HttpRequest,
    form: web::Json<DownloadProj>,
) -> actix_web::Result<impl actix_web::Responder> {
    info!("start download project");
    let path = handle_compress_proj_async(form.into_inner())
        .await
        .map_err(|e| {
            log::error!("compress project panicked: {:?}", e);
            actix_web::error::ErrorInternalServerError("compress failed")
        })?;
    let file = tokio::task::spawn_blocking(move || NamedFile::open(&path))
        .await
        .map_err(|e| {
            log::error!("open archive panicked: {:?}", e);
            actix_web::error::ErrorInternalServerError("open archive failed")
        })?;
    match file {
        Ok(file) => {
            info!("process complete");
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

async fn upload_full_project_output(
    MultipartForm(form): MultipartForm<ProjFullUploadFile>,
) -> HttpResponse {
    save_full_proj_output(form).await
}

 pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/inner-tex/project")
        .route("/download", web::put().to(download_project))
        .route("/upload-output", web::post().to(upload_project_output))
        .route("/upload-full-output", web::post().to(upload_full_project_output))
    );
    // configure multipart limits for this inner upload endpoint
    let inner_upload_config = MultipartFormConfig::default()
        .total_limit(104857600) // 100 MB
        .memory_limit(209715200);
    cfg.service(web::scope("/inner-tex/ul/proj").app_data(inner_upload_config));
}
