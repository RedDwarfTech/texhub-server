use crate::diesel::RunQueryDsl;
use crate::{
    common::database::get_connection,
    model::{
        diesel::tex::custom_tex_models::TexProjFolder,
        request::project::query::snippet_query_params::SnippetQueryParams,
    },
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl};
use log::error;
use rust_wheel::model::user::login_user_info::LoginUserInfo;

pub async fn get_snippets(params: web::Query<SnippetQueryParams>, login_user_info: LoginUserInfo) {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    let cvs = query.load::<TexProjFolder>(&mut get_connection());
    match cvs {
        Ok(result) => {}
        Err(err) => {
            error!("get proj folder failed, {}", err);
        }
    }
}
