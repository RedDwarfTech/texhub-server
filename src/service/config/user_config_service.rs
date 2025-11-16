use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::conf::user_conf_add::TexUserConfigAdd;
use crate::{
    common::database::get_connection, model::diesel::tex::custom_tex_models::TexUserConfig,
};
use diesel::upsert::on_constraint;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
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

pub fn get_user_config_by_key(uid: &i64, key: String) -> Option<TexUserConfig> {
    use crate::model::diesel::tex::tex_schema::tex_user_config as user_config_table;
    let mut query = user_config_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(
        user_config_table::user_id
            .eq(uid)
            .and(user_config_table::config_key.eq(key)),
    );

    match query.first::<TexUserConfig>(&mut get_connection()) {
        Ok(conf) => Some(conf),
        Err(diesel::result::Error::NotFound) => None,
        Err(err) => {
            error!("Failed to get user config,{}", err);
            None
        }
    }
}

pub fn save_user_config(uid: &i64, key: &String, value: &String) {
    use crate::model::diesel::tex::tex_schema::tex_user_config::dsl::*;
    let new_conf = TexUserConfigAdd::gen_user_config(uid, value, key);
    diesel::insert_into(tex_user_config)
        .values(&new_conf)
        .on_conflict(on_constraint("tex_user_config_unique"))
        .do_update()
        .set((config_value.eq(&new_conf.config_value),))
        .get_result::<TexUserConfig>(&mut get_connection())
        .expect("failed to add new conf");
}
