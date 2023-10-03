use std::ffi::CString;

use actix_web::{HttpResponse, web};

use crate::synctex::{synctex_scanner_new_with_output_file, synctex_display_query, synctex_node_page, synctex_node_visible_height, synctex_node_p, synctex_scanner_next_result, synctex_node_visible_h, synctex_node_visible_width, synctex_node_visible_v};


async fn get_pdf_position() -> HttpResponse {
    let proj_dir = "/Users/xiaoqiangjiang/source/reddwarf/backend/rust-learn";
    let file_path = "/Users/xiaoqiangjiang/source/reddwarf/backend/rust-learn/tex/demo.pdf";
    unsafe {
        let c_file_path = CString::new(file_path.clone());
        if let Err(e) = c_file_path {
            println!("parse out path error,{},{}", e, file_path.clone());
            return HttpResponse::Ok().json("ok");
        }
        let c_build_path = CString::new(proj_dir.clone());
        if let Err(e) = c_build_path {
            println!("parse build path error,{},{}", e, proj_dir.clone());
            return HttpResponse::Ok().json("ok");
        }
        let scanner = synctex_scanner_new_with_output_file(
            c_file_path.unwrap().as_ptr(),
            c_build_path.unwrap().as_ptr(),
            1,
        );
        println!("c result: {:?}", scanner);
        let demo_tex =
            CString::new("/Users/xiaoqiangjiang/source/reddwarf/backend/rust-learn/tex/demo.tex");
        let node_number = synctex_display_query(scanner, demo_tex.unwrap().as_ptr(), 1, 1, 0);
        if node_number > 0 {
            for i in 0..node_number {
                let node: synctex_node_p = synctex_scanner_next_result(scanner);
                println!("node...{:?}", node);
                let page = synctex_node_page(node);
                println!("page: {:?}", page);
                let h = synctex_node_visible_h(node);
                println!("h: {:?}", h);
                let v = synctex_node_visible_v(node);
                println!("v: {:?}", v);
                let height = synctex_node_visible_height(node);
                println!("height: {:?}", height);
                let width = synctex_node_visible_width(node);
                println!("width: {:?}", width);
            }
        }
        println!("node_number result: {:?}", node_number);
    }
    HttpResponse::Ok().json("ok")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/project")
            .route("/pos/pdf",web::get().to(get_pdf_position))
            
    );
}