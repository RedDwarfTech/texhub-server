use crate::diesel::RunQueryDsl;
use crate::{
    common::database::get_connection, model::diesel::tex::custom_tex_models::TexUserConfig,
};
use diesel::{ExpressionMethods, QueryDsl};
use log::error;

pub fn get_user_config(uid: &i64) -> Option<Vec<TexUserConfig>> {
    use crate::model::diesel::tex::tex_schema::tex_user_config as user_config_table;
    let mut query = user_config_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(user_config_table::user_id.eq(uid));
    let files = query.load::<TexUserConfig>(&mut get_connection());
    if let Err(err) = files {
        error!("Failed to get user config,{}", err);
        return None;
    }
    return Some(files.unwrap());
}
