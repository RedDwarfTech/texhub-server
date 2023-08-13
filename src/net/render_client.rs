use log::{error, info};
use reqwest::Client;
use rust_wheel::config::app::app_conf_reader::get_app_config;

pub async fn render_request(file_path: &String, out_path: &String) {
    let client = Client::new();
    let url_path = format!("{}", "/render/compile/v1/project/");
    let url = format!("{}{}", get_app_config("render.render_api_url"), url_path);
    let json_data = serde_json::json!({
        "file_path": file_path,
        "out_path": out_path,
    });
    let response = client.post(url).json(&json_data).send().await;
    match response {
        Ok(r) => {
            let resp:Result<serde_json::Value, reqwest::Error> = r.json().await;
            match resp {
                Ok(content) => {
                    let result = content.get("result").unwrap();
                    info!("compile request result,{}", result);
                },
                Err(e) => {
                    error!("get response failed: {}",e);
                },
            }
            
            //let r: serde_json::Value = r.json().await.unwrap();
            //let result = r.get("result").unwrap();
            //info!("compile request result,{}", result)
        }
        Err(e) => error!("request compile error: {}", e),
    }
}
