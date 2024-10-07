use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

use crate::{
    common::database::get_connection,
    model::{
        app::{tex_file_params::TexFileParams, tpl_params::ProjDynParams},
        dict::role_type::RoleType,
        diesel::{
            custom::{
                file::file_add::TexFileAdd,
                project::{folder::folder_map_add::FolderMapAdd, tex_project_add::TexProjectAdd},
            },
            tex::custom_tex_models::{TexFile, TexProjFolder, TexProjFolderMap, TexProject},
        },
        request::project::{
            add::{tex_folder_req::TexFolderReq, tex_project_req::TexProjectReq},
            edit::edit_proj_folder::EditProjFolder,
        },
    },
    service::{
        global::proj::proj_util::get_proj_base_dir_instant,
        project::{
            eden::external_app::init_project_into_yjs,
            project_editor_service::create_proj_editor,
            project_folder_map_service::move_proj_folder,
            project_folder_service::{create_proj_default_folder, get_proj_default_folder},
        },
    },
};
use diesel::{result::Error, Connection, PgConnection, RunQueryDsl};
use log::error;
use rust_wheel::{
    common::{
        infra::user::rd_user::get_user_info,
        util::rd_file_util::{copy_dir_recursive, create_directory_if_not_exists},
    },
    model::user::{login_user_info::LoginUserInfo, rd_user_info::RdUserInfo},
    texhub::th_file_type::ThFileType,
};
use tokio::task;

pub async fn create_project_dyn_params(
    tpl_params: &ProjDynParams,
    login_user_info: &LoginUserInfo,
) -> Result<Option<TexProject>, Error> {
    // check the folder main.tex file
    let full_path = Path::new(&tpl_params.tpl_files_dir).join(tpl_params.main_file_name.clone());
    if !full_path.exists() {
        error!(
            "did not found main file, full path:{}, tpl params:{:?}",
            full_path.to_string_lossy(),
            tpl_params
        );
        return Ok(None);
    }
    let user_info: RdUserInfo = get_user_info(&login_user_info.userId).await.unwrap();
    let mut connection = get_connection();
    let trans_result: Result<Option<TexProject>, Error> = connection.transaction(|connection| {
        return do_create_tpl_proj_trans(&tpl_params, &user_info, connection, login_user_info);
    });
    return trans_result;
}

fn do_create_tpl_proj_trans(
    tpl_params: &ProjDynParams,
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
    login_user_info: &LoginUserInfo,
) -> Result<Option<TexProject>, Error> {
    let proj_req: TexProjectReq = TexProjectReq {
        name: tpl_params.name.to_string(),
        template_id: Some(tpl_params.tpl_id.to_owned()),
        folder_id: None,
        legacy_proj_id: None,
        proj_source_type: Some(tpl_params.proj_source_type.to_owned()),
        proj_source: Some(tpl_params.proj_source.clone()),
    };
    let create_result = create_proj(&proj_req, connection, rd_user_info);
    if let Err(ce) = create_result {
        error!("Failed to create proj: {}", ce);
        return Err(ce);
    }
    let proj = create_result.unwrap();
    do_create_proj_dependencies(&proj_req, rd_user_info, connection, &proj);
    do_create_proj_on_disk(tpl_params, &proj, rd_user_info, login_user_info);
    return Ok(Some(proj));
}

pub fn do_create_proj_on_disk(
    tpl_params: &ProjDynParams,
    proj: &TexProject,
    rd_user_info: &RdUserInfo,
    login_user_info: &LoginUserInfo,
) {
    let create_res = create_proj_files(
        tpl_params,
        &proj.project_id,
        &rd_user_info.id,
        login_user_info,
    );
    if !create_res {
        error!(
            "create project files failed,tpl: {:?}, project: {:?}",
            tpl_params, proj
        );
        return;
    }
}

pub fn create_proj_files(
    tpl_params: &ProjDynParams,
    proj_id: &String,
    uid: &i64,
    login_user_info: &LoginUserInfo,
) -> bool {
    let proj_dir = get_proj_base_dir_instant(&proj_id);
    match create_directory_if_not_exists(&proj_dir) {
        Ok(()) => {}
        Err(e) => error!("create project directory before tpl copy failed,{}", e),
    }
    let result = copy_dir_recursive(&tpl_params.tpl_files_dir.as_str(), &proj_dir);
    if let Err(e) = result {
        error!(
            "copy file failed,{}, tpl dir: {}, project dir: {}",
            e, tpl_params.tpl_files_dir, proj_dir
        );
        return false;
    }
    return create_files_into_db(
        &proj_dir,
        proj_id,
        uid,
        &tpl_params.main_file_name,
        login_user_info,
    );
}

pub fn create_proj(
    proj_req: &TexProjectReq,
    connection: &mut PgConnection,
    rd_user_info: &RdUserInfo,
) -> Result<TexProject, diesel::result::Error> {
    let uid: i64 = rd_user_info.id;
    let new_proj = TexProjectAdd::from_req(&proj_req, &uid, &rd_user_info.nickname);
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let result = diesel::insert_into(tex_project)
        .values(&new_proj)
        .get_result::<TexProject>(connection);
    return result;
}

pub fn create_files_into_db(
    project_path: &String,
    proj_id: &String,
    uid: &i64,
    main_name: &String,
    login_user_info: &LoginUserInfo,
) -> bool {
    let mut files: Vec<TexFileAdd> = Vec::new();
    let tex_file_params = TexFileParams {
        proj_id: proj_id.to_owned(),
        main_name: main_name.to_owned(),
        user_id: uid.to_owned(),
    };
    let read_result = read_directory(&tex_file_params, project_path, proj_id, &mut files);
    if let Err(err) = read_result {
        error!(
            "read directory failed,{}, project path: {}",
            err, project_path
        );
        return false;
    }
    use crate::model::diesel::tex::tex_schema::tex_file as files_table;
    if files.len() == 0 {
        error!(
            "read 0 files from disk, project path: {}, main_file_name: {:?}",
            project_path, main_name
        );
        return false;
    }
    let result = diesel::insert_into(files_table::dsl::tex_file)
        .values(&files)
        .get_result::<TexFile>(&mut get_connection());
    if let Err(err) = result {
        error!("write files into db facing issue,{}", err);
        return false;
    }
    let u_copy = login_user_info.clone();
    task::spawn_blocking({
        move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(init_project_into_yjs(&files, &u_copy));
        }
    });
    return true;
}

/**
 * scan the directory file and save into database
 * ignore some unused files
 */
fn read_directory(
    params: &TexFileParams,
    dir_path: &str,
    parent_id: &str,
    files: &mut Vec<TexFileAdd>,
) -> io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        if let Err(err) = entry {
            error!(
                "read directory entry failed, {}, dir path: {}, parent: {}",
                err, dir_path, parent_id
            );
            return Err(err);
        }
        let entry: DirEntry = entry?;
        let file_name: OsString = entry.file_name();
        let path = entry.path();
        let proj_path = get_proj_base_dir_instant(&params.proj_id);
        let relative_path = path.parent().unwrap().strip_prefix(proj_path);
        let stored_path = relative_path.unwrap().to_string_lossy().into_owned();
        if path.is_file() {
            handle_proj_files(params, stored_path, &file_name, parent_id, files, path);
        } else if path.is_dir() {
            let tex_file = TexFileAdd::gen_tex_file_from_disk(
                stored_path,
                &params.user_id,
                &params.proj_id.to_owned(),
                &file_name,
                &params.main_name,
                parent_id,
                ThFileType::Folder as i32,
            );
            let parent_folder_id = tex_file.file_id.clone();
            files.push(tex_file);
            let dir_name = file_name.to_string_lossy().into_owned();
            let next_parent = format!("{}/{}", dir_path, dir_name);
            let recur_result = read_directory(params, &next_parent, &parent_folder_id, files);
            if let Err(err) = recur_result {
                error!(
                    "read file failed, {}, next parant: {}, dir path: {}",
                    err, next_parent, dir_path
                );
            }
        }
    }

    Ok(())
}

fn handle_proj_files(
    params: &TexFileParams,
    stored_path: String,
    file_name: &OsString,
    parent_id: &str,
    files: &mut Vec<TexFileAdd>,
    path: PathBuf,
) {
    let extension = path.extension().and_then(|ext| ext.to_str());
    let ignored_extensions: HashSet<&str> = ["log", "aux", "out", "toc", "nav", "snm", "vrb"]
        .iter()
        .cloned()
        .collect();
    let ignored_files: HashSet<&str> = [".DS_Store"].iter().cloned().collect();
    if let Some(ext) = extension {
        // ignore some system aux files that do not need to show in projects tree
        if ignored_extensions.contains(ext) {
            return;
        }
    }
    if ignored_files.contains(&file_name.clone().into_string().unwrap().as_str()) {
        return;
    }
    let tex_file = TexFileAdd::gen_tex_file_from_disk(
        stored_path,
        &params.user_id,
        &params.proj_id,
        file_name,
        &params.main_name,
        parent_id,
        1,
    );
    files.push(tex_file);
}

pub fn do_create_proj_dependencies(
    proj_req: &TexProjectReq,
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
    proj: &TexProject,
) {
    let default_folder: TexProjFolder = create_default_folder(rd_user_info, connection, &proj);
    let edit_req: EditProjFolder = EditProjFolder {
        project_id: proj.project_id.clone(),
        folder_id: if proj_req.folder_id.is_some() {
            proj_req.folder_id.unwrap()
        } else {
            default_folder.id
        },
        proj_type: 1,
    };
    let uid: i64 = rd_user_info.id;
    move_proj_folder(&edit_req, &uid, connection);
    let editor_result = create_proj_editor(
        &proj.project_id.clone(),
        rd_user_info,
        RoleType::Owner as i32,
        connection,
    );
    if let Err(e) = editor_result {
        error!("create editor facing issue, error: {}", e)
    }
}

fn create_default_folder(
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
    proj: &TexProject,
) -> TexProjFolder {
    let default_folder = get_proj_default_folder(rd_user_info, connection);
    if default_folder.is_none() {
        let default_add = TexFolderReq {
            folder_name: "default".to_owned(),
            proj_type: 1,
            default_folder: 1,
        };
        let new_default_folder = create_proj_default_folder(connection, rd_user_info, &default_add);
        let uid: i64 = rd_user_info.id;
        let map_add = EditProjFolder {
            proj_type: 1,
            project_id: proj.project_id.clone(),
            folder_id: new_default_folder.id,
        };
        let new_folder_map = FolderMapAdd::from_req(&map_add, &uid);
        use crate::model::diesel::tex::tex_schema::tex_proj_folder_map::dsl::*;
        diesel::insert_into(tex_proj_folder_map)
            .values(&new_folder_map)
            .get_result::<TexProjFolderMap>(connection)
            .expect("add default folder map failed");
        return new_default_folder;
    }
    return default_folder.unwrap();
}
