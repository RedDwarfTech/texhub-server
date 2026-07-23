use crate::diesel::RunQueryDsl;
use crate::model::dict::tex_file_compile_status::TeXFileCompileStatus;
use crate::model::diesel::tex::custom_tex_models::TexCompQueue;
use crate::{
    common::database::get_connection,
    model::request::project::queue::queue_req::QueueReq,
    model::request::project::queue::queue_start_time_req::QueueStartTimeReq,
};
use actix_web::HttpResponse;
use chrono::{Duration, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, QueryDsl, QueryResult};
use log::error;
use rust_wheel::{
    common::{
        util::time_util::get_current_millisecond,
        wrapper::actix_http_resp::{box_actix_rest_response, box_error_actix_rest_response},
    },
    model::user::login_user_info::LoginUserInfo,
    texhub::proj::compile_result::CompileResult,
};

pub fn get_queue_by_id(queue_id: &i64) -> Option<TexCompQueue> {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue as comp_queue_table;
    let query = comp_queue_table::table.into_boxed::<diesel::pg::Pg>();
    let record: QueryResult<TexCompQueue> = query
        .filter(comp_queue_table::id.eq(queue_id))
        .first::<TexCompQueue>(&mut get_connection());
    match record {
        Ok(rec) => {
            return Some(rec);
        }
        Err(diesel::result::Error::NotFound) => {
            return None;
        }
        Err(e) => {
            error!("search queue by id error {}, id: {}", e, queue_id);
            return None;
        }
    }
}

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

    // build query
    let mut query = folder_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(folder_table::comp_status.eq(TeXFileCompileStatus::Compiling as i32));
    query = query.filter(folder_table::created_time.lt(et));

    // execute query
    let cvs_result = query.load::<TexCompQueue>(&mut get_connection());

    let cvs = match cvs_result {
        Ok(list) => list,
        Err(e) => {
            error!(
                "update_expired_proj_queue: failed to load compile queue: {}",
                e
            );
            return;
        }
    };

    if !cvs.is_empty() {
        let ids: Vec<i64> = cvs.iter().map(|q| q.id).collect();
        let predicate = comp_status
            .eq(TeXFileCompileStatus::Compiling as i32)
            .and(id.eq_any(&ids));

        match diesel::update(tex_comp_queue.filter(predicate))
            .set(comp_status.eq(TeXFileCompileStatus::Expired as i32))
            .execute(&mut get_connection())
        {
            Ok(_count) => {}
            Err(e) => error!("update_expired_proj_queue: failed to update rows: {}", e),
        }
    }
}

/// 取项目近 3 次成功编译耗时的最大值（毫秒），无有效数据时返回 0。
pub fn get_estimated_compile_time(project_id: &str) -> i64 {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue as comp_queue_table;
    let records: QueryResult<Vec<Option<i64>>> = comp_queue_table::table
        .select(comp_queue_table::compile_duration_ms)
        .filter(comp_queue_table::project_id.eq(project_id))
        .filter(comp_queue_table::comp_result.eq(CompileResult::Success as i32))
        .filter(comp_queue_table::comp_status.eq(TeXFileCompileStatus::Compiled as i32))
        .filter(comp_queue_table::compile_duration_ms.is_not_null())
        .order_by(comp_queue_table::complete_time.desc())
        .limit(3)
        .load::<Option<i64>>(&mut get_connection());
    match records {
        Ok(list) => list.into_iter().flatten().filter(|cost| *cost > 0).max().unwrap_or(0),
        Err(e) => {
            error!(
                "get estimated compile time failed, project_id: {}, error: {}",
                project_id, e
            );
            0
        }
    }
}

pub fn update_queue_start_time(params: &QueueStartTimeReq) -> HttpResponse {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue::dsl::*;
    use crate::service::project::proj::project_service::cache_queue;

    let predicate = id.eq(params.id);
    let start_time_value = if params.start_time > 0 {
        params.start_time
    } else {
        get_current_millisecond()
    };
    let update_result = diesel::update(tex_comp_queue.filter(predicate))
        .set((
            start_time.eq(start_time_value),
            updated_time.eq(get_current_millisecond()),
        ))
        .get_result::<TexCompQueue>(&mut get_connection());
    if let Err(e) = update_result {
        let err_msg = format!(
            "update compile queue start time failed, error info:{}, params:{:?}",
            e, params
        );
        error!("{}", err_msg);
        return box_error_actix_rest_response("", "UPDATE_QUEUE_START_TIME_FAILED".to_owned(), err_msg);
    }
    let queue = update_result.unwrap();
    if let Some(resp) = cache_queue(&queue) {
        return resp;
    }
    box_actix_rest_response(queue)
}
