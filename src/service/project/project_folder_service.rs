use crate::model::diesel::tex::custom_tex_models::TexProjFolder;
use crate::model::request::project::add::tex_folder_req::TexFolderReq;
use crate::{diesel::RunQueryDsl, model::diesel::custom::project::folder::folder_add::FolderAdd};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use rust_wheel::model::user::rd_user_info::RdUserInfo;

pub fn get_proj_default_folder(
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
) -> Option<TexProjFolder> {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder as folder_table;
    let mut query = folder_table::table.into_boxed::<diesel::pg::Pg>();
    let uid: i64 = rd_user_info.id;
    query = query.filter(folder_table::user_id.eq(uid));
    query = query.filter(folder_table::default_folder.eq(1));
    query = query.filter(folder_table::proj_type.eq(1));
    let cvs: Vec<TexProjFolder> = query
        .load::<TexProjFolder>(connection)
        .expect("Failed to get default folder");
    if cvs.len() == 1 {
        return Some(cvs[0].clone());
    }
    if cvs.len() == 0 {
        return None;
    }
    return Some(cvs[0].clone());
}

pub fn create_proj_default_folder(
    connection: &mut PgConnection,
    rd_user_info: &RdUserInfo,
    folder_add: &TexFolderReq,
) -> TexProjFolder {
    let uid: i64 = rd_user_info.id;
    let new_proj = FolderAdd::from_req(folder_add, &uid);
    use crate::model::diesel::tex::tex_schema::tex_proj_folder::dsl::*;
    let result = diesel::insert_into(tex_proj_folder)
        .values(&new_proj)
        .get_result::<TexProjFolder>(connection)
        .expect("create default folder facing issue");
    return result;
}
