use chrono::Duration;
use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::file::file_add::TexFileAdd;
use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::model::request::file::file_add_req::TexFileAddReq;
use crate::model::request::file::file_del::TexFileDelReq;
use crate::model::response::file::file_tree_resp::FileTreeResp;
use crate::service::project::project_service::del_project_file;
use diesel::result::Error;
use diesel::{sql_query, Connection, ExpressionMethods, PgConnection, QueryDsl};
use log::error;
use rust_wheel::common::util::convert_to_tree_generic::convert_to_tree;
use rust_wheel::common::util::model_convert::map_entity;
use rust_wheel::config::cache::redis_util::{sync_get_str, set_value};

pub fn get_file_by_fid(filter_id: &String) -> TexFile {
    let cached_file = sync_get_str(&filter_id).unwrap();
    if cached_file.is_some() {
        let tf:TexFile = serde_json::from_str(&cached_file.unwrap()).unwrap();
        return tf;
    }
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::file_id.eq(filter_id));
    let files = query.load::<TexFile>(&mut get_connection()).unwrap();
    let file = &files[0];
    let file_json = serde_json::to_string(file).unwrap();
    let one_day = Duration::days(1);
    let seconds_in_one_day = one_day.num_seconds();
    set_value(&filter_id, &file_json, seconds_in_one_day as usize).unwrap();
    return file.to_owned();
}

pub fn get_file_list(parent_id: &String) -> Vec<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::parent.eq(parent_id));
    let cvs = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn create_file(add_req: &TexFileAddReq) -> TexFile {
    let new_file = TexFileAdd::gen_tex_file(add_req);
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let result = diesel::insert_into(tex_file)
        .values(&new_file)
        .get_result::<TexFile>(&mut get_connection())
        .expect("failed to add new tex file or folder");
    return result;
}

pub fn delete_file_recursive(del_req: &TexFileDelReq) -> Result<usize, Error> {
    let mut connection = get_connection();
    let trans_result = connection.transaction(|connection| {
        let delete_result = del_single_file(&del_req.file_id, connection);
        match delete_result {
            Ok(proj) => {
                del_project_file(&del_req.file_id, connection);
                return Ok(proj);
            }
            Err(e) => diesel::result::QueryResult::Err(e),
        }
    });
    return trans_result;
}

pub fn del_single_file(
    del_file_id: &String,
    connection: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let predicate = crate::model::diesel::tex::tex_schema::tex_file::file_id.eq(del_file_id);
    let delete_result = diesel::delete(tex_file.filter(predicate)).execute(connection);
    return delete_result;
}

pub fn get_file_tree(parent_id: &String) -> Vec<FileTreeResp> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::parent.eq(parent_id));
    let cvs = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return find_sub_menu_cte_impl(&result, parent_id);
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn find_sub_menu_cte_impl(_root_menus: &Vec<TexFile>, root_id: &String) -> Vec<FileTreeResp> {
    let mut connection = get_connection();
    let cte_query_sub_menus = format!(
        " with recursive sub_files as (
        select 
          id, 
          name, 
          file_id, 
          sort,
          created_time,
          updated_time,
          user_id,
          doc_status,
          project_id,
          file_type,
          parent,
          main_flag 
        from 
          tex_file mr 
        where 
          parent = '{}' 
        union all 
        select 
          origin.id, 
          origin.name, 
          origin.file_id, 
          origin.sort,
          origin.created_time,
          origin.updated_time,
          origin.user_id,
          origin.doc_status,
          origin.project_id,
          origin.file_type,
          origin.parent,
          origin.main_flag 
        from 
          sub_files 
          join tex_file origin on origin.parent = sub_files.file_id
      ) 
      select 
        id, 
        name, 
        file_id, 
        sort,
        created_time,
        updated_time,
        user_id,
        doc_status,
        project_id,
        file_type,
        parent,
        main_flag 
      from 
        sub_files 
      order by 
        sort asc;      
    ",
        root_id
    );
    let cte_menus = sql_query(cte_query_sub_menus)
        .load::<TexFile>(&mut connection)
        .expect("Error find file");
    let menu_resource_resp: Vec<FileTreeResp> = map_entity(cte_menus);
    return convert_to_tree_impl(&menu_resource_resp, root_id);
}

fn convert_to_tree_impl(contents: &Vec<FileTreeResp>, root_id: &str) -> Vec<FileTreeResp> {
    let root_element: Vec<_> = contents
        .iter()
        .filter(|content| content.parent == root_id)
        .collect();
    let sub_element: Vec<_> = contents
        .iter()
        .filter(|content| content.parent != root_id)
        .collect();
    let result = convert_to_tree(&root_element, &sub_element);
    return result;
}
