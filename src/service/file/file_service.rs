use std::fs::File;
use std::io::{Read, Write};

use crate::common::database::get_connection;
use crate::controller::file::file_controller::FileCodeParams;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::file::file_add::TexFileAdd;
use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::model::request::file::file_add_req::TexFileAddReq;
use crate::model::request::file::file_del::TexFileDelReq;
use crate::model::request::file::file_rename::TexFileRenameReq;
use crate::model::response::file::file_tree_resp::FileTreeResp;
use crate::service::project::project_service::del_project_file;
use actix_web::HttpResponse;
use chrono::Duration;
use diesel::result::Error;
use diesel::{
    sql_query, BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl,
};
use log::error;
use rust_wheel::common::util::convert_to_tree_generic::convert_to_tree;
use rust_wheel::common::util::model_convert::map_entity;
use rust_wheel::common::util::rd_file_util::{create_folder_not_exists, join_paths};
use rust_wheel::common::wrapper::actix_http_resp::{
    box_actix_rest_response, box_error_actix_rest_response,
};
use rust_wheel::config::app::app_conf_reader::get_app_config;
use rust_wheel::config::cache::redis_util::{set_value, sync_get_str};
use rust_wheel::model::user::login_user_info::LoginUserInfo;

pub fn get_file_by_fid(filter_id: &String) -> Option<TexFile> {
    let cached_file = sync_get_str(&filter_id).unwrap();
    if cached_file.is_some() {
        let tf: TexFile = serde_json::from_str(&cached_file.unwrap()).unwrap();
        return Some(tf);
    }
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::file_id.eq(filter_id));
    let files = query.load::<TexFile>(&mut get_connection()).unwrap();
    if files.len() == 0 {
        return None;
    }
    let file = &files[0];
    let file_json = serde_json::to_string(file).unwrap();
    let one_day = Duration::days(1);
    let seconds_in_one_day = one_day.num_seconds();
    set_value(&filter_id, &file_json, seconds_in_one_day as usize).unwrap();
    return Some(file.to_owned());
}

pub fn get_file_list(parent_id: &String) -> Vec<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::parent.eq(parent_id));
    let cvs: Result<Vec<TexFile>, Error> = query.load::<TexFile>(&mut get_connection());
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

pub fn get_main_file_list(project_id: &String) -> Option<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(
        cv_work_table::project_id
            .eq(project_id)
            .and(cv_work_table::main_flag.eq(1)),
    );
    let cvs: Result<Vec<TexFile>, Error> = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return Some(result[0].to_owned());
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return None;
        }
    }
}

pub fn get_text_file_code(filter_file_id: &String) -> String {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::file_id.eq(filter_file_id));
    let cvs: Result<Vec<_>, Error> = query.load::<TexFile>(&mut get_connection());
    let tex_files: Vec<TexFile> = cvs.unwrap();
    let tex = tex_files[0].clone();
    let file_folder = format!("{}/{}", base_compile_dir, tex.project_id);
    let file_full_path = format!("{}/{}", file_folder, tex.name);
    let mut file = match File::open(file_full_path) {
        Ok(file) => file,
        Err(error) => {
            error!("Error opening file: {:?}", error);
            return "".to_string();
        }
    };
    let mut contents = String::new();
    if let Err(error) = file.read_to_string(&mut contents) {
        error!("Error reading file: {:?}", error);
        return "".to_string();
    }
    return contents;
}

pub fn create_file(add_req: &TexFileAddReq, login_user_info: &LoginUserInfo) -> HttpResponse {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(
        cv_work_table::parent
            .eq(add_req.parent.clone())
            .and(cv_work_table::name.eq(add_req.name.clone()))
            .and(cv_work_table::file_type.eq(add_req.file_type.clone())),
    );
    let cvs = query.load::<TexFile>(&mut get_connection()).unwrap();
    if !cvs.is_empty() {
        return box_error_actix_rest_response(
            "already exists",
            "ALREADY_EXISTS".to_owned(),
            "file/folder already exists".to_owned(),
        );
    }
    let gen_file_path = get_file_path(add_req);
    let new_file = TexFileAdd::gen_tex_file(add_req, login_user_info, &gen_file_path);
    let result = diesel::insert_into(tex_file)
        .values(&new_file)
        .get_result::<TexFile>(&mut get_connection())
        .expect("failed to add new tex file or folder");
    create_file_on_disk(&result);
    let resp = box_actix_rest_response(result);
    return resp;
}

pub fn create_file_on_disk(file: &TexFile) {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let split_path = &[
        base_compile_dir,
        file.project_id.clone(),
        file.file_path.clone(),
        file.name.clone(),
    ];
    let file_full_path = join_paths(split_path);
    if file.file_type == 0 {
        create_folder_not_exists(&file_full_path);
    } else {
        let create_result = create_disk_file(&file_full_path);
        if let Err(e) = create_result {
            error!("create file on disk failed, {}", e);
        }
    }
}

fn create_disk_file(file_path: &str) -> std::io::Result<()> {
    if !std::path::Path::new(file_path).exists() {
        let mut file = File::create(file_path)?;
        file.write_all(b"")?;
    }
    Ok(())
}

pub fn get_file_path(add_req: &TexFileAddReq) -> String {
    let file_info = get_file_by_fid(&add_req.parent);
    if file_info.is_none() {
        return "/".to_owned();
    }
    let finfo = file_info.unwrap();
    if add_req.file_type == 0 {
        return format!("{}{}/", finfo.file_path.clone(), finfo.name.clone());
    } else {
        return finfo.file_path.clone();
    }
}

pub fn file_init_complete(edit_req: &FileCodeParams) -> TexFile {
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let predicate =
        crate::model::diesel::tex::tex_schema::tex_file::file_id.eq(edit_req.file_id.clone());
    let update_result = diesel::update(tex_file.filter(predicate))
        .set(yjs_initial.eq(1))
        .get_result::<TexFile>(&mut get_connection())
        .expect("unable to update tex file");
    return update_result;
}

pub fn rename_file_impl(edit_req: &TexFileRenameReq, login_user_info: &LoginUserInfo) -> TexFile {
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let predicate = crate::model::diesel::tex::tex_schema::tex_file::file_id
        .eq(edit_req.file_id.clone())
        .and(crate::model::diesel::tex::tex_schema::tex_file::user_id.eq(login_user_info.userId));
    let update_result = diesel::update(tex_file.filter(predicate))
        .set(name.eq(edit_req.name.clone()))
        .get_result::<TexFile>(&mut get_connection())
        .expect("unable to update tex file name");
    return update_result;
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
          main_flag,
          yjs_initial,
          file_path 
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
          origin.main_flag,
          origin.yjs_initial,
          origin.file_path 
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
        main_flag,
        yjs_initial,
        file_path 
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
