use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::file::file_ver_add::TexFileVersionAdd;
use crate::model::request::file::add::file_add_ver_req::TexFileVerAddReq;
use crate::model::request::project::query::file_version_params_v1::FileVersionParamsV1;
use crate::{
    common::database::get_connection,
    model::{
        diesel::tex::custom_tex_models::TexFileVersion,
        request::project::query::file_version_params::FileVersionParams,
    },
};
use diesel::result::Error;
use diesel::{ExpressionMethods, QueryDsl};
use log::error;
use rust_wheel::config::app::app_conf_reader::get_app_config;
use rust_wheel::model::user::login_user_info::LoginUserInfo;

/**
 * https://discuss.yjs.dev/t/for-versioning-should-i-store-snapshot-or-document-copies/2421
 */
pub fn create_file_ver(
    add_req: &TexFileVerAddReq,
    login_user_info: &LoginUserInfo,
) -> Option<TexFileVersion> {
    use crate::model::diesel::tex::tex_schema::tex_file_version::dsl::*;
    let new_file = TexFileVersionAdd::gen_tex_file_version(add_req, login_user_info);

    let result = diesel::insert_into(tex_file_version)
        .values(&new_file)
        .get_result::<TexFileVersion>(&mut get_connection());
    match result {
        Ok(_) => {
            return Some(result.unwrap());
        }
        Err(err) => {
            error!("add file version failed, {}", err);
            return None;
        }
    }
}

pub fn get_latest_file_version_by_fid(fid: &str) -> Option<TexFileVersion> {
    use crate::model::diesel::tex::tex_schema::tex_file_version as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::file_id.eq(fid));
    query = query.order_by(cv_work_table::created_time.desc());
    let cvs: Result<TexFileVersion, Error> = query.first::<TexFileVersion>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return Some(result);
        }
        Err(err) => {
            error!("get file version failed, {},fid:{}", err, fid);
            return None;
        }
    }
}

pub fn update_file_version(edit_req: &TexFileVerAddReq, update_id: &i64) -> Option<TexFileVersion> {
    use crate::model::diesel::tex::tex_schema::tex_file_version as oper_table;
    use oper_table::dsl::*;
    let predicate = oper_table::id.eq(update_id);
    let update_result = diesel::update(tex_file_version.filter(predicate))
        .set((
            content.eq(edit_req.content.clone()),
            snapshot.eq(edit_req.snapshot.clone()),
        ))
        .get_result::<TexFileVersion>(&mut get_connection())
        .expect("unable to update tex file version");
    return Some(update_result);
}

pub fn update_version_status(update_id: &i64) -> Option<TexFileVersion> {
    use crate::model::diesel::tex::tex_schema::tex_file_version as oper_table;
    use oper_table::dsl::*;
    let predicate = oper_table::id.eq(update_id);
    let update_result = diesel::update(tex_file_version.filter(predicate))
        .set(version_status.eq(1))
        .get_result::<TexFileVersion>(&mut get_connection())
        .expect("unable to update tex file version");
    return Some(update_result);
}

pub fn get_proj_history(
    history_req: &FileVersionParams,
    login_user_info: &LoginUserInfo,
) -> Option<TexFileVersion> {
    use crate::model::diesel::tex::tex_schema::tex_file_version as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::id.eq(history_req.id.clone()));
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    let files = query.first::<TexFileVersion>(&mut get_connection());
    match files {
        Ok(rec) => {
            return Some(rec);
        }
        Err(diesel::result::Error::NotFound) => {
            return None;
        }
        Err(e) => {
            error!("get file snapshot error {}", e);
            return None;
        }
    }
}

pub async fn get_proj_history_v1(
    history_req: &FileVersionParamsV1,
) -> Option<TexFileVersion> {
    let client = reqwest::Client::new();
    let base_url = get_app_config("texhub.y_websocket_api_url");
    let url = format!(
        "{}/doc/version/proj/scroll/detail?id={}",
        base_url.trim_end_matches('/'),
        history_req.id,
    );
    
    let resp = client.get(&url).send().await;
    if let Err(e) = &resp {
        error!("get_proj_history_v1: http request failed, error: {:?}", e);
        return None;
    }
    
    let r = resp.unwrap();
    let s = match r.text().await {
        Ok(text) => text,
        Err(e) => {
            error!("get_proj_history_v1: failed to get response text, error: {:?}", e);
            return None;
        }
    };
    
    let json = match serde_json::from_str::<serde_json::Value>(&s) {
        Ok(value) => value,
        Err(e) => {
            error!("get_proj_history_v1: json parse error: {:?}, raw: {}", e, s);
            return None;
        }
    };
    
    let data = match json.get("result") {
        Some(d) => d,
        None => {
            error!("get_proj_history_v1: 'data' field missing, json: {:?}", json);
            return None;
        }
    };
    
    match serde_json::from_value::<TexFileVersion>(data.clone()) {
        Ok(version) => Some(version),
        Err(e) => {
            error!("get_proj_history_v1: parse TexFileVersion failed: {:?}, json: {:?}", e, data);
            None
        }
    }
}