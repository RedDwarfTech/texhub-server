use crate::diesel::RunQueryDsl;
use crate::model::diesel::tex::custom_tex_models::TexSnippet;
use crate::{
    common::database::get_connection,
    model::request::project::query::snippet_query_params::SnippetQueryParams,
};
use diesel::{ExpressionMethods, QueryDsl};
use log::error;
use rust_wheel::model::user::login_user_info::LoginUserInfo;

pub async fn get_snippets(
    params: SnippetQueryParams,
    login_user_info: &LoginUserInfo,
) -> Vec<TexSnippet> {
    use crate::model::diesel::tex::tex_schema::tex_snippet as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    let cvs = query.load::<TexSnippet>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get snip failed, {}", err);
            return Vec::new();
        }
    }
}
