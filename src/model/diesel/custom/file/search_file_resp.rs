use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SearchFileResp {
    pub name: String,
    pub file_id: String,
    pub file_path: String,
    pub content: String,
}

impl SearchFileResp {
    pub(crate) fn new_file(file_map: Map<String,Value>) -> Self {
        let f_name = format!("{}",file_map.get("name").unwrap().to_string().trim_matches('"'));
        let f_file_id = format!("{}",file_map.get("file_id").unwrap().to_string().trim_matches('"'));
        let f_path = format!("{}",file_map.get("file_path").unwrap().to_string().trim_matches('"'));
        Self {
            name: f_name,
            file_id: f_file_id,
            file_path: f_path,
            content: file_map.get("content").unwrap().to_string()
        }
    }
}
