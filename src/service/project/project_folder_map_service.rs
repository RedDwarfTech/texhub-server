use crate::{
    common::database::get_connection,
    model::{
        diesel::custom::project::folder::folder_map_add::FolderMapAdd,
        request::project::edit::edit_proj_folder::EditProjFolder,
    },
};
use diesel::{upsert::on_constraint, ExpressionMethods};
use crate::diesel::RunQueryDsl;

pub fn move_proj_folder(
    edit_req: &EditProjFolder,
    uid: &i64,
) {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder_map::dsl::*;
    let add_map = FolderMapAdd::from_req(edit_req, uid);
    let insert_result = diesel::insert_into(tex_proj_folder_map)
        .values(&add_map)
        .on_conflict(on_constraint("tex_proj_folder_map_user_proj_un"))
        .do_update()
        .set(folder_id.eq(edit_req.folder_id))
        .execute(&mut get_connection())
        .expect("move to the folder failed");
}
