use crate::{model::request::project::query::download_proj::DownloadProj, service::project::project_service::handle_compress_proj};
use actix_files::NamedFile;
use actix_web::{HttpRequest, web};
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/inner-tex/project").route("/download", web::put().to(download_project)));
}
