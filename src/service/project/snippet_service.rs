use crate::diesel::RunQueryDsl;
use crate::model::diesel::tex::custom_tex_models::TexSnippet;
use crate::model::request::snippet::edit::snippet_req::SnippetReq;
use crate::{
    common::database::get_connection,
    model::request::project::query::snippet_query_params::SnippetQueryParams,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
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

pub fn del_snippet_impl(
    del_id: &i64,
    login_user_info: &LoginUserInfo,
) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_snippet as tex_project_table;
    let predicate = tex_project_table::id
        .eq(del_id)
        .and(tex_project_table::user_id.eq(login_user_info.userId));
    let delete_result = diesel::delete(tex_project_table::dsl::tex_snippet.filter(predicate))
        .execute(&mut get_connection());
    return delete_result;
}

pub fn edit_snippet_impl(edit_req: &SnippetReq, login_user_info: &LoginUserInfo) -> TexSnippet {
    use crate::model::diesel::tex::tex_schema::tex_snippet as tex_file_table;
    use tex_file_table::dsl::*;
    let predicate = tex_file_table::id
        .eq(edit_req.id.clone())
        .and(tex_file_table::user_id.eq(login_user_info.userId));
    let update_msg = format!(
        "unable to update tex file name, user id: {}, file_id: {}",
        login_user_info.userId,
        edit_req.id.clone()
    );
    let update_result = diesel::update(tex_snippet.filter(predicate))
        .set(snippet.eq(edit_req.name.clone()))
        .get_result::<TexSnippet>(&mut get_connection())
        .expect(&update_msg);
    return update_result;
}

pub fn add_snippet_impl(edit_req: &SnippetReq, login_user_info: &LoginUserInfo) -> TexSnippet {
    use crate::model::diesel::tex::tex_schema::tex_snippet as tex_file_table;
    use tex_file_table::dsl::*;
    let update_result = diesel::insert_into(tex_snippet)
        .values(snippet.eq(edit_req.name.clone()))
        .get_result::<TexSnippet>(&mut get_connection())
        .expect("add snippet failed");
    return update_result;
}