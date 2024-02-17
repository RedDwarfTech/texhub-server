use crate::{common::database::get_connection, model::request::project::queue::queue_req::QueueReq};
use crate::diesel::RunQueryDsl;
use crate::model::diesel::tex::custom_tex_models::TexCompQueue;
use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use log::error;
use rust_wheel::model::user::login_user_info::LoginUserInfo;

pub fn get_proj_queue_list(req: &QueueReq,login_user_info: &LoginUserInfo) -> Vec<TexCompQueue> {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue as comp_queue_table;
    let mut query = comp_queue_table::table.into_boxed::<diesel::pg::Pg>();
    if !req.comp_status.is_empty() {
        query = query.filter(comp_queue_table::comp_result.eq_any(req.comp_status.clone()));
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

pub fn get_latest_proj_queue(req: &Vec<i32>,uid: &i64, project_id: &String) -> Option<TexCompQueue> {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue as comp_queue_table;
    let mut query = comp_queue_table::table.into_boxed::<diesel::pg::Pg>();
    if !req.is_empty() {
        query = query.filter(comp_queue_table::comp_result.eq_any(req.clone()));
    }
    query = query.filter(comp_queue_table::project_id.eq(project_id));
    query = query.filter(comp_queue_table::user_id.eq(uid));
    query = query.order_by(comp_queue_table::created_time.desc());
    let newest_record: QueryResult<TexCompQueue> = query
        .first::<TexCompQueue>(&mut get_connection());
    match newest_record {
        Ok(rec) => {
            return Some(rec);
        },
        Err(diesel::result::Error::NotFound) => {
            return None;
        },
        Err(e) => {
            error!("search newest queue error {}, uid: {}, project id: {}", e, uid, project_id);
            return None;
        },
    }
}