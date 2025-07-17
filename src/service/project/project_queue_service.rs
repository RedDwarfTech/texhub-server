use crate::diesel::RunQueryDsl;
use crate::model::dict::tex_file_compile_status::TeXFileCompileStatus;
use crate::model::diesel::tex::custom_tex_models::TexCompQueue;
use crate::{
    common::database::get_connection, model::request::project::queue::queue_req::QueueReq,
};
use chrono::{Duration, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, QueryResult};
use log::{error, info};
use rust_wheel::model::user::login_user_info::LoginUserInfo;

pub fn get_proj_working_queue_list(
    req: &QueueReq,
    login_user_info: &LoginUserInfo,
) -> Vec<TexCompQueue> {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue as comp_queue_table;
    let mut query = comp_queue_table::table.into_boxed::<diesel::pg::Pg>();
    if !req.comp_status.is_empty() {
        query = query.filter(comp_queue_table::comp_status.eq_any(req.comp_status.clone()));
    }
    if !req.project_id.is_empty() {
        query = query.filter(comp_queue_table::project_id.eq(req.project_id.clone()));
    }
    query = query.filter(comp_queue_table::user_id.eq(login_user_info.userId));
    let cvs: Vec<TexCompQueue> = query
        .load::<TexCompQueue>(&mut get_connection())
        .expect("Failed to get queue");
    return cvs;
}

pub fn get_latest_proj_queue(
    req: &Vec<i32>,
    uid: &i64,
    project_id: &String,
) -> Option<TexCompQueue> {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue as comp_queue_table;
    let mut query = comp_queue_table::table.into_boxed::<diesel::pg::Pg>();
    if !req.is_empty() {
        query = query.filter(comp_queue_table::comp_result.eq_any(req.clone()));
    }
    query = query.filter(comp_queue_table::project_id.eq(project_id));
    query = query.filter(comp_queue_table::user_id.eq(uid));
    query = query.order_by(comp_queue_table::created_time.desc());
    let newest_record: QueryResult<TexCompQueue> =
        query.first::<TexCompQueue>(&mut get_connection());
    match newest_record {
        Ok(rec) => {
            return Some(rec);
        }
        Err(diesel::result::Error::NotFound) => {
            return None;
        }
        Err(e) => {
            error!(
                "search newest queue error {}, uid: {}, project id: {}",
                e, uid, project_id
            );
            return None;
        }
    }
}

pub fn update_expired_proj_queue() {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue::dsl::*;
    let expire_time = Utc::now() + Duration::minutes(10);
    use crate::model::diesel::tex::tex_schema::tex_comp_queue as folder_table;
    let et = expire_time.timestamp_millis();
    let mut query = folder_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(folder_table::comp_status.eq(TeXFileCompileStatus::Compiling as i32));
    query = query.filter(folder_table::created_time.lt(et));
    let cvs: Vec<TexCompQueue> = query
        .load::<TexCompQueue>(&mut get_connection())
        .expect("Failed to get compile queue");
    if !cvs.is_empty() {
        info!("update record{}", et);
        let ids: Vec<i64> = cvs.iter().map(|q| q.id).collect();
        let predicate = comp_status
            .eq(TeXFileCompileStatus::Compiling as i32)
            .and(id.eq_any(&ids));
        let _ = diesel::update(tex_comp_queue.filter(predicate))
            .set(comp_status.eq(TeXFileCompileStatus::Expired as i32))
            .execute(&mut get_connection());
    } else {
        info!("no compiling record,et:{}", et);
    }
}
