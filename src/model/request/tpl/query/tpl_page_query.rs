#[derive(serde::Deserialize)]
pub struct TplPageQuery {
    pub name: Option<String>,
    pub tpl_type: Option<i32>,
}