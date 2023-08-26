use crate::controller::project::project_controller::{EditPrjReq, GetPrjParams};
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::file::file_add::TexFileAdd;
use crate::model::diesel::custom::project::tex_proj_editor_add::TexProjEditorAdd;
use crate::model::diesel::custom::project::tex_project_add::TexProjectAdd;
use crate::model::diesel::tex::custom_tex_models::{TexProjEditor, TexProject};
use crate::model::request::project::tex_compile_project_req::TexCompileProjectReq;
use crate::model::request::project::tex_join_project_req::TexJoinProjectReq;
use crate::net::render_client::render_request;
use crate::{common::database::get_connection, model::diesel::tex::custom_tex_models::TexFile};
use diesel::result::Error;
use diesel::{sql_query, Connection, ExpressionMethods, PgConnection, QueryDsl};
use log::{error, warn};
use rust_wheel::config::app::app_conf_reader::get_app_config;
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use std::fs::{self, File};
use std::io::{self, Write};

pub fn get_prj_list(_tag: &String, login_user_info: &LoginUserInfo) -> Vec<TexProject> {
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    let cvs = query.load::<TexProject>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get docs failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_prj_by_id(proj_id: &String) -> TexProject {
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::project_id.eq(proj_id));
    let cvs = query.load::<TexProject>(&mut get_connection()).unwrap();
    return cvs[0].clone();
}

pub fn edit_proj(edit_req: &EditPrjReq) -> TexProject {
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let predicate = crate::model::diesel::tex::tex_schema::tex_project::project_id
        .eq(edit_req.project_id.clone());
    let update_result = diesel::update(tex_project.filter(predicate))
        .set(proj_name.eq(&edit_req.proj_name))
        .get_result::<TexProject>(&mut get_connection())
        .expect("unable to update tex project");
    return update_result;
}

pub fn create_empty_project(proj_name: &String, user_id: &i64) -> Result<TexProject, Error> {
    let mut connection = get_connection();
    let trans_result = connection.transaction(|connection| {
        let create_result = create_proj(proj_name, connection, user_id);
        match create_result {
            Ok(proj) => {
                let result = create_main_file(&proj.project_id, connection, user_id);
                match result {
                    Ok(_) => {
                        create_main_file_on_disk(&proj.project_id);
                    }
                    Err(e) => {
                        error!("create file failed,{}", e)
                    }
                }
                return Ok(proj);
            }
            Err(e) => diesel::result::QueryResult::Err(e),
        }
    });
    return trans_result;
}

fn create_main_file_on_disk(project_id: &String) {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let file_folder = format!("{}/{}", base_compile_dir, project_id);
    match create_directory_if_not_exists(&file_folder) {
        Ok(()) => {}
        Err(e) => error!("create directory failed,{}", e),
    }
    if let Ok(mut file) = File::create(format!("{}/{}", &file_folder, "main.tex")) {
        if let Err(we) = file.write_all(b"Hello, World!") {
            error!("write content failed, {}", we);
        }
    } else {
        error!("create file failed");
    }
}

fn create_directory_if_not_exists(path: &str) -> io::Result<()> {
    if !fs::metadata(path).is_ok() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn create_main_file(
    proj_id: &String,
    connection: &mut PgConnection,
    uid: &i64,
) -> Result<TexFile, diesel::result::Error> {
    let new_proj = TexFileAdd::gen_tex_main(proj_id, uid);
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let result = diesel::insert_into(tex_file)
        .values(&new_proj)
        .get_result::<TexFile>(connection);
    return result;
}

fn create_proj(
    name: &String,
    connection: &mut PgConnection,
    uid: &i64,
) -> Result<TexProject, diesel::result::Error> {
    let new_proj = TexProjectAdd::from_req(name, &uid);
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let result = diesel::insert_into(tex_project)
        .values(&new_proj)
        .get_result::<TexProject>(connection);
    return result;
}

pub fn join_project(
    req: &TexJoinProjectReq,
    login_user_info: &LoginUserInfo,
) -> Result<TexProjEditor, Error> {
    let new_proj_editor = TexProjEditorAdd::from_req(&req.project_id, &login_user_info.userId);
    use crate::model::diesel::tex::tex_schema::tex_proj_editor::dsl::*;
    let result = diesel::insert_into(tex_proj_editor)
        .values(&new_proj_editor)
        .get_result::<TexProjEditor>(&mut get_connection());
    return result;
}

pub fn del_project(del_project_id: &String) {
    let mut connection = get_connection();
    let result = connection.transaction(|connection| {
        let delete_result = del_project_impl(del_project_id, connection);
        match delete_result {
            Ok(rows) => {
                if rows == 0 {
                    warn!(
                        "the delete project effect {} rows, project id: {}",
                        rows, del_project_id
                    );
                }
                del_project_file(del_project_id, connection);
                Ok("")
            }
            Err(e) => diesel::result::QueryResult::Err(e),
        }
    });
    match result {
        Ok(_) => {}
        Err(e) => {
            error!(
                "transaction failed, project id: {},error:{}",
                del_project_id, e
            );
        }
    }
}

pub fn del_project_impl(
    del_project_id: &String,
    connection: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let predicate =
        crate::model::diesel::tex::tex_schema::tex_project::project_id.eq(del_project_id);
    let delete_result = diesel::delete(tex_project.filter(predicate)).execute(connection);
    return delete_result;
}

pub fn del_project_file(del_project_id: &String, connection: &mut PgConnection) {
    let del_command = format!(
        "WITH RECURSIVE x AS (
            SELECT file_id
            FROM   tex_file
            WHERE parent = '{}'
        
            UNION  ALL
            SELECT a.file_id
            FROM   x
            JOIN   tex_file a ON a.parent = x.file_id
            )
         DELETE FROM tex_file a
         USING  x
         WHERE a.file_id = x.file_id",
        del_project_id
    );
    let cte_menus = sql_query(&del_command).load::<TexFile>(connection);
    match cte_menus {
        Ok(_) => {}
        Err(e) => {
            error!(
                "delete project file failed, project id: {}, command:{},error info:{}",
                del_project_id, del_command, e
            );
        }
    }
}

pub async fn compile_project(params: &TexCompileProjectReq) -> Option<serde_json::Value> {
    let prj = get_prj_by_id(&params.project_id);
    return render_request(params, &prj).await;
}

pub async fn get_project_pdf(params: &GetPrjParams) -> String {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let prj_dir = format!("{}/{}", base_compile_dir, params.project_id);
    if !fs::metadata(&prj_dir).is_ok() {
        error!("folder did not exists, dir: {}", prj_dir);
        return "".to_owned();
    }
    let subdirectories = fs::read_dir(prj_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    // https://stackoverflow.com/questions/76946130/creation-time-is-not-available-on-this-platform-currently
    let latest_directory = subdirectories
        .iter()
        .max_by_key(|&dir| dir.metadata().unwrap().modified().unwrap());

    match latest_directory {
        Some(directory) => {
            let name = directory.file_name().unwrap_or_default().to_str().unwrap();
            return name.to_string();
        }
        None => {
            return "".to_owned();
        }
    }
}
