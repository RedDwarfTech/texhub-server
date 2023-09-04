use crate::controller::project::project_controller::get_temp_auth_code;
use actix_web::{test, web, App};

#[actix_rt::test]
async fn test_compile_see() {
    let mut app =
        test::init_service(App::new().route("/", web::get().to(get_temp_auth_code))).await;
    let req = test::TestRequest::get().to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}
