#[cfg(test)]
mod tests {
    use crate::controller::project::proj::proj_controller::sse_handler;
    use actix_web::{test, web, App};

    #[actix_rt::test]
    async fn test_proj_sse() {
        let mut app = test::init_service(App::new().route("/", web::get().to(sse_handler))).await;
        let req = test::TestRequest::get()
            .uri("/?project_id=c98f73bc869143d084eb38a0fc38a8e7&req_time=1234&file_name=main.tex")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        let status = resp.status().is_success();
        let body = test::read_body(resp).await;
        println!("Response body: {}", String::from_utf8_lossy(&body));
        assert!(status);
    }
}
