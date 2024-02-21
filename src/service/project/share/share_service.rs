use crate::common::database::get_connection;
use crate::model::diesel::tex::custom_tex_models::TexProjEditor;
use crate::{
    diesel::RunQueryDsl, model::request::project::query::share_query_params::ShareQueryParams,
};
use diesel::{ExpressionMethods, QueryDsl};
use log::error;
use rust_wheel::model::user::login_user_info::LoginUserInfo;

pub async fn get_collar_users(
    params: ShareQueryParams,
    login_user_info: &LoginUserInfo,
) -> Vec<TexProjEditor> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    let cvs = query.load::<TexProjEditor>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get collarboration user failed, {}", err);
            return Vec::new();
        }
    }
}
