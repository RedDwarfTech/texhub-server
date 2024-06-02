use crate::common::database::get_connection;
use crate::model::dict::role_type::RoleType;
use crate::model::diesel::tex::custom_tex_models::TexProjEditor;
use crate::model::request::project::share::share_del::ShareDel;
use crate::{
    diesel::RunQueryDsl, model::request::project::query::share_query_params::ShareQueryParams,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, QueryResult};
use log::error;
use rust_wheel::model::user::login_user_info::LoginUserInfo;

pub async fn get_collar_users(params: &ShareQueryParams) -> Vec<TexProjEditor> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::project_id.eq(params.project_id.clone()));
    query = query.filter(cv_work_table::role_id.eq(RoleType::Collarboartor as i32));
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

pub fn del_share_bind_impl(params: &ShareDel, login_user_info: &LoginUserInfo) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as editor_table;
    // check login user, only the project owner could delete the bind relationship
    let mut query = editor_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(editor_table::project_id.eq(params.project_id.clone()));
    query = query.filter(editor_table::role_id.eq(RoleType::Owner as i32));
    let editor:QueryResult<TexProjEditor> = query.first::<TexProjEditor>(&mut get_connection());
    match editor {
        Ok(rec) => {
            if rec.user_id == login_user_info.userId {
                return do_delete(params);
            }
            return Err(diesel::result::Error::NotFound);
        }
        Err(diesel::result::Error::NotFound) => {
            return Err(diesel::result::Error::NotFound);
        }
        Err(e) => {
            error!("get file snapshot error {}", e);
            return Err(e);
        }
    }
}

fn do_delete(params: &ShareDel) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as editor_table;
    let predicate = editor_table::id
        .eq(params.id)
        .eq(editor_table::project_id.eq(params.project_id.clone()))
        .and(editor_table::role_id.ne(RoleType::Owner as i32));
    let delete_result = diesel::delete(editor_table::dsl::tex_proj_editor.filter(predicate))
        .execute(&mut get_connection());
    return delete_result;
}