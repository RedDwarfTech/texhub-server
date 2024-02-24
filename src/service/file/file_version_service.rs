use diesel::{ExpressionMethods, QueryDsl};
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use crate::diesel::RunQueryDsl;
use crate::{
    common::database::get_connection,
    model::{
        diesel::tex::custom_tex_models::TexFileVersion,
        request::project::query::file_version_params::FileVersionParams,
    },
};

pub fn get_proj_history(
    history_req: &FileVersionParams,
    login_user_info: &LoginUserInfo,
) -> Vec<TexFileVersion> {
    use crate::model::diesel::tex::tex_schema::tex_file_version as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::id.eq(history_req.id.clone()));
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    let files: Vec<TexFileVersion> = query
        .load::<TexFileVersion>(&mut get_connection())
        .expect("get project version facing error");
    return files;
}
