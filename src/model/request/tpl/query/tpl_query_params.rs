#[derive(serde::Deserialize)]
pub struct TplQueryParams {
    pub name: Option<String>,
    pub tpl_type: Option<i32>,
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}