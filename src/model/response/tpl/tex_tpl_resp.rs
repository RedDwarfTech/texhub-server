use serde::{Deserialize, Serialize};
use crate::model::diesel::tex::custom_tex_models::TexTemplate;

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TexTplResp {
    pub id: i64,
    pub name: String,
    pub preview_url: Option<String>,
    pub template_id: i64,
}

impl From<&TexTemplate> for TexTplResp {
    fn from(tpl: &TexTemplate) -> Self {
        Self { 
            id: tpl.id,
            name: tpl.name.clone(),
            preview_url: tpl.preview_url.clone(),
            template_id: tpl.template_id
        }
    }
}
