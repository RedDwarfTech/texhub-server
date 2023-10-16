use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::project::tex_project_add::TexProjectAdd;
use crate::model::diesel::tex::custom_tex_models::{TexProject, TexTemplate};
use crate::model::request::tpl::query::tpl_query_params::TplQueryParams;
use diesel::{ExpressionMethods, QueryDsl, TextExpressionMethods, QueryResult};
use log::error;
use rust_wheel::common::query::pagination::Paginate;
use rust_wheel::common::util::model_convert::map_pagination_res;
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::model::response::pagination_response::PaginationResponse;
use uuid::Uuid;

pub fn get_tpl_list(params: &TplQueryParams) -> Vec<TexTemplate> {
    use crate::model::diesel::tex::tex_schema::tex_template as cv_tpl_table;
    let mut query = cv_tpl_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_tpl_table::online_status.eq(1));
    if params.name.as_ref().is_some() {
        query =
            query.filter(cv_tpl_table::name.like(format!("%{}%", params.name.as_ref().unwrap())));
    }
    if params.tpl_type.as_ref().is_some() {
        query = query.filter(cv_tpl_table::template_type.eq(params.tpl_type.as_ref().unwrap()));
    }
    let cvs = query.load::<TexTemplate>(&mut get_connection());
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

pub fn get_tpl_page_impl(params: &TplQueryParams) -> PaginationResponse<Vec<TexTemplate>> {
    use crate::model::diesel::tex::tex_schema::tex_template as cv_tpl_table;
    let mut query = cv_tpl_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_tpl_table::online_status.eq(1));
    if params.name.as_ref().is_some() {
        query =
            query.filter(cv_tpl_table::name.like(format!("%{}%", params.name.as_ref().unwrap())));
    }
    if params.tpl_type.as_ref().is_some() {
        let tpl_type_tmp = params.tpl_type.clone().unwrap();
        query = query.filter(cv_tpl_table::template_type.eq(tpl_type_tmp));
    }
    let query = query
        .paginate(params.page_num.unwrap_or(1).clone())
        .per_page(params.page_size.unwrap_or(9).clone());
    let page_result:QueryResult<(Vec<TexTemplate>, i64, i64)> = query.load_and_count_pages_total::<TexTemplate>(&mut get_connection());
    let page_map_result = map_pagination_res(
        page_result,
        params.page_num.unwrap_or(1),
        params.page_size.unwrap_or(10),
    );
    return page_map_result;
}

pub fn create_tpl(input_doc: &String) -> TexProject {
    let uuid = Uuid::new_v4();
    let uuid_string = uuid.to_string().replace("-", "");
    let new_doc = TexProjectAdd {
        proj_name: input_doc.to_string(),
        created_time: get_current_millisecond(),
        updated_time: get_current_millisecond(),
        user_id: 1,
        proj_status: 1,
        template_id: 1,
        project_id: uuid_string,
        nickname: "default".to_string()
    };
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;

    let result = diesel::insert_into(tex_project)
        .values(&new_doc)
        .get_result::<TexProject>(&mut get_connection())
        .expect("get insert doc failed");
    return result;
}

pub fn get_tempalte_by_id(tpl_id: &i64) -> TexTemplate {
    use crate::model::diesel::tex::tex_schema::tex_template as cv_tpl_table;
    let mut query = cv_tpl_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_tpl_table::template_id.eq(tpl_id));
    let tpl = query
        .load::<TexTemplate>(&mut get_connection())
        .expect("error get template by id");
    return tpl.get(0).unwrap().to_owned();
}

