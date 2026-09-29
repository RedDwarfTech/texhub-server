use crate::common::database::get_connection;
use crate::model::dict::collar_status::CollarStatus;
use crate::model::dict::role_type::RoleType;
use crate::model::diesel::tex::custom_tex_models::TexProjEditor;
use crate::model::request::project::share::collar_query_params::CollarQueryParams;
use crate::model::request::project::share::share_del::ShareDel;
use crate::model::response::project::share::collar_permission_resp::CollarPermissionResp;
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

pub async fn get_collar_relation(params: &CollarQueryParams) -> Option<Vec<TexProjEditor>> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::project_id.eq(params.project_id.clone()));
    query = query.filter(cv_work_table::user_id.eq(params.user_id));
    query = query.filter(cv_work_table::collar_status.eq(CollarStatus::Normal as i32));
    let cvs = query.load::<TexProjEditor>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return Some(result);
        }
        Err(err) => {
            error!("get collarboration user failed, {}", err);
            return Some(Vec::new());
        }
    }
}

/// 查询某个用户对某个项目的协作权限，供 texhub-broadcast 在 WS 建连时做
/// membership 校验。
///
/// 身份以显式入参传入而非请求结构体：调用方只能决定「查哪个项目」，
/// 「查谁」必须来自 AuthMiddleware 校验过的 JWT。把它做成结构体字段曾经
/// 出过一次事故（字段必填但调用方不传，接口固定 400）。
///
/// 与 `get_collar_relation` 的区别在于**失败语义**：这里 DB 出错时返回
/// `Err`（调用方必须 fail-closed 拒绝连接），而不是像前者那样吞掉错误返回
/// 空列表 —— 空列表在鉴权语境下等于"没有权限"，但把"查不到"当成"查到了
/// 没有人"会让数据库抖动直接变成越权。
pub async fn get_collar_permission(
    project_id: &str,
    user_id: i64,
) -> Result<CollarPermissionResp, String> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::project_id.eq(project_id.to_owned()));
    query = query.filter(cv_work_table::user_id.eq(user_id));
    query = query.filter(cv_work_table::collar_status.eq(CollarStatus::Normal as i32));
    let cvs = query
        .load::<TexProjEditor>(&mut get_connection())
        .map_err(|err| {
            let msg = format!("get collar permission failed, {}", err);
            error!("{}", msg);
            msg
        })?;

    // 一行都没有 => 非成员（或已被移出协作）。多条时取 Owner 优先。
    let role_id = cvs
        .iter()
        .map(|item| item.role_id)
        .filter(|role| *role == RoleType::Owner as i32 || *role == RoleType::Collarboartor as i32)
        .min()
        .unwrap_or(0);
    let is_member = role_id != 0;

    Ok(CollarPermissionResp {
        project_id: project_id.to_owned(),
        user_id,
        role_id,
        is_member,
        // Owner 与 Collaborator 目前都可写；只读角色尚未建模，见 can_read_only 后续拆分
        can_write: is_member,
    })
}

pub fn del_share_bind_impl(params: &ShareDel, login_user_info: &LoginUserInfo) -> Result<usize, diesel::result::Error> {    use crate::model::diesel::tex::tex_schema::tex_proj_editor as editor_table;
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