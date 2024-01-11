use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::project::tex_proj_editor_add::TexProjEditorAdd;
use crate::{
    common::database::get_connection, model::diesel::tex::custom_tex_models::TexProjEditor,
};
use diesel::{ExpressionMethods, QueryDsl};
use rust_wheel::model::user::rd_user_info::RdUserInfo;

pub fn get_default_proj_ids(uid: i64, proj_ids: &Vec<String>) -> Vec<String> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as folder_table;
    let mut query = folder_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(folder_table::user_id.eq(uid));
    query = query.filter(folder_table::project_id.eq_any(proj_ids));
    query = query.filter(folder_table::proj_status.eq(1));
    let cvs: Vec<TexProjEditor> = query
        .load::<TexProjEditor>(&mut get_connection())
        .expect("Failed to get default folder");
    let proj_ids: Vec<String> = cvs.iter().map(|item| item.project_id.clone()).collect();
    return proj_ids;
}

pub fn create_proj_editor(
    proj_id: &String,
    rd_user_info: &RdUserInfo,
    rid: i32,
) -> Result<TexProjEditor, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as proj_editor_table;
    let uid: i64 = rd_user_info.id.parse().unwrap();
    let proj_editor = TexProjEditorAdd::from_req(proj_id, &uid, rid);
    let result = diesel::insert_into(proj_editor_table::dsl::tex_proj_editor)
        .values(&proj_editor)
        .get_result::<TexProjEditor>(&mut get_connection());
    return result;
}