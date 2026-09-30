use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::tex::custom_tex_models::{
    TexFile, TexFileVersion, TexProjEditor, TexProject,
};
use crate::model::dict::collar_status::CollarStatus;
use crate::model::dict::role_type::RoleType;
use crate::model::error::texhub_error::TexhubError;
use crate::service::file::file_service::get_cached_file_by_fid;
use crate::service::project::project_queue_service::get_queue_by_id;
use diesel::{ExpressionMethods, QueryDsl};
use log::error;

/// 读接口的项目归属守卫。
///
/// 背景：相当一批读接口（文件树、源码、PDF、项目详情、历史版本、编译日志…）
/// 以前只按请求里的 id 直接查数据，既不校验调用者身份，也不校验他对目标项目
/// 的权限；其中一部分甚至连 `LoginUserInfo` 都没声明，等于完全公开。这里把
/// 「解析出目标项目 + 校验成员身份」收口到一处，读接口在返回数据前必须先过
/// 它，避免以后新增路由时又漏掉。
///
/// 三条原则：
///
/// 1. **身份来自 AuthMiddleware**。调用方必须传已经验签的 `user_id`，绝不接受
///    请求参数里的 user_id。
/// 2. **fail-closed**。归属解析不出来、DB 报错，都判定为拒绝。把「查不到」
///    当成「查到了且有权限」会让一次数据库抖动变成一次越权。
/// 3. **不误伤项目创建者**。`tex_proj_editor` 里的 Owner 行是在建项目后
///    非事务写入、失败只记日志的（见 `eden::proj::do_create_proj_dependencies`），
///    所以「查不到协作行」不等于「不是主人」。所有权以 `tex_project.user_id`
///    为准，协作表只用来判定 Collaborator。
///
/// 判定通过返回 `Ok(())`，否则返回具体的 `TexhubError`，由 controller 用
/// 既有的 `box_err_actix_rest_response` 转成响应（HTTP 200 + resultCode）。
pub fn ensure_project_readable(project_id: &str, user_id: i64) -> Result<(), TexhubError> {
    if project_id.trim().is_empty() {
        return Err(TexhubError::ProjAccessUnresolvable);
    }

    // 权威所有者：tex_project.user_id
    match get_proj_owner_id(project_id) {
        Ok(Some(owner_id)) => {
            if owner_id == user_id {
                return Ok(());
            }
            // 项目确实存在但不属于当前用户，继续看是不是 Collaborator。
        }
        Ok(None) => {
            // 项目不存在时无法判定归属，直接拒绝，不区分「不存在」和「无权限」。
            return Err(TexhubError::ProjAccessUnresolvable);
        }
        Err(e) => {
            // 查不出来不等于有权限，fail-closed。
            return Err(e);
        }
    }

    match get_collar_role(project_id, user_id) {
        Ok(role_id) => {
            if role_id == RoleType::Owner as i32 || role_id == RoleType::Collarboartor as i32 {
                Ok(())
            } else {
                Err(TexhubError::ProjAccessDenied)
            }
        }
        Err(e) => Err(e),
    }
}

/// 查协作行上的角色；0 表示没有有效的协作关系。
/// DB 出错返回 Err，让调用方 fail-closed。
fn get_collar_role(project_id: &str, user_id: i64) -> Result<i32, TexhubError> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as editor_table;
    let mut query = editor_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(editor_table::project_id.eq(project_id.to_owned()));
    query = query.filter(editor_table::user_id.eq(user_id));
    query = query.filter(editor_table::collar_status.eq(CollarStatus::Normal as i32));
    let rows = query
        .load::<TexProjEditor>(&mut get_connection())
        .map_err(|err| {
            error!("proj_access_guard: load collar row failed, err={}", err);
            TexhubError::ProjAccessCheckFailed
        })?;

    // 多行时 Owner 优先（Owner=1 < Collaborator=2，取 min）。
    let role_id = rows
        .iter()
        .map(|item| item.role_id)
        .filter(|role| *role == RoleType::Owner as i32 || *role == RoleType::Collarboartor as i32)
        .min()
        .unwrap_or(0);
    Ok(role_id)
}

/// 查项目所有者。
///
/// 刻意不复用 `proj::project_service::get_prj_by_id`：那个函数对 diesel 结果
/// 直接 `.unwrap()`，DB 抖动时会 panic。鉴权路径上 panic 不可接受 —— 至少要
/// 能干净地返回 403。
///
/// 返回 `Ok(None)` 表示项目不存在，`Err` 表示查询失败。
fn get_proj_owner_id(project_id: &str) -> Result<Option<i64>, TexhubError> {
    use crate::model::diesel::tex::tex_schema::tex_project as proj_table;
    let query = proj_table::table.into_boxed::<diesel::pg::Pg>();
    let rows = query
        .filter(proj_table::project_id.eq(project_id.to_owned()))
        .limit(1)
        .load::<TexProject>(&mut get_connection())
        .map_err(|err| {
            error!("proj_access_guard: load project failed, err={}", err);
            TexhubError::ProjAccessCheckFailed
        })?;
    Ok(rows.first().map(|p| p.user_id))
}

/// 按 `file_id` 解析所属项目，再校验权限。
pub fn ensure_file_readable(file_id: &str, user_id: i64) -> Result<(), TexhubError> {
    let project_id = resolve_project_by_file_id(file_id)?;
    ensure_project_readable(&project_id, user_id)
}

/// `parent` 既可能是 project_id（顶层，前端传 projectId），也可能是父目录的
/// file_id（子层级，前端传 pid）。两种都要能解析出项目。
pub fn ensure_file_tree_readable(parent: &str, user_id: i64) -> Result<(), TexhubError> {
    let project_id = resolve_project_by_tree_parent(parent)?;
    ensure_project_readable(&project_id, user_id)
}

/// 按编译队列 id 解析所属项目，再校验权限。
pub fn ensure_queue_readable(queue_id: i64, user_id: i64) -> Result<(), TexhubError> {
    let project_id = match get_queue_by_id(&queue_id) {
        Some(queue) => queue.project_id,
        None => {
            return Err(TexhubError::ProjAccessUnresolvable);
        }
    };
    ensure_project_readable(&project_id, user_id)
}

/// 按文件版本行 id 解析所属项目，再校验权限。
pub fn ensure_file_version_readable(version_id: i64, user_id: i64) -> Result<(), TexhubError> {
    let project_id = match get_file_version_project_id(version_id) {
        Some(pid) => pid,
        None => {
            return Err(TexhubError::ProjAccessUnresolvable);
        }
    };
    ensure_project_readable(&project_id, user_id)
}

fn resolve_project_by_file_id(file_id: &str) -> Result<String, TexhubError> {
    // 优先走缓存版本；它命中时不碰 DB。
    if let Some(file) = get_cached_file_by_fid(&file_id.to_string()) {
        return Ok(file.project_id);
    }
    // 未命中。`get_cached_file_by_fid` 内部对 DB 结果 `.unwrap()`，DB 抖动会
    // 直接 panic；鉴权路径上不能把 worker 炸掉，所以这里自己查一次可失败
    // 的版本。查不到 => 无法判定归属 => 拒绝。
    use crate::model::diesel::tex::tex_schema::tex_file as file_table;
    let query = file_table::table.into_boxed::<diesel::pg::Pg>();
    let rows = query
        .filter(file_table::file_id.eq(file_id.to_owned()))
        .limit(1)
        .load::<TexFile>(&mut get_connection())
        .map_err(|err| {
            error!("proj_access_guard: load file failed, err={}", err);
            TexhubError::ProjAccessCheckFailed
        })?;
    match rows.first() {
        Some(f) => Ok(f.project_id.clone()),
        None => Err(TexhubError::ProjAccessUnresolvable),
    }
}

/// 先按 project_id 试（顶层根节点），再按 file_id 试（子目录）。
/// 顺序不能反：file_id 与 project_id 是不同命名空间，但万一某个 file_id 恰好
/// 等于另一个项目的 project_id，先查项目会命中错误的项目。
fn resolve_project_by_tree_parent(parent: &str) -> Result<String, TexhubError> {
    match get_proj_owner_id(parent) {
        // 命中说明它确实是个 project_id，直接用。
        Ok(Some(_)) => return Ok(parent.to_string()),
        // 查不到可能是「不是项目」也可能是「DB 出错」。后者不能当成
        // 「不是项目」继续往 file_id 分支走，否则会把 DB 故障伪装成一次
        // 正常的 404。Err 直接上抛。
        Err(e) => return Err(e),
        Ok(None) => {}
    }
    resolve_project_by_file_id(parent)
}

fn get_file_version_project_id(version_id: i64) -> Option<String> {
    use crate::model::diesel::tex::tex_schema::tex_file_version as version_table;
    let query = version_table::table.into_boxed::<diesel::pg::Pg>();
    let record = query
        .filter(version_table::id.eq(version_id))
        .first::<TexFileVersion>(&mut get_connection());
    match record {
        Ok(rec) => Some(rec.project_id),
        Err(diesel::result::Error::NotFound) => None,
        Err(e) => {
            error!("proj_access_guard: get file version failed, err={}", e);
            None
        }
    }
}
