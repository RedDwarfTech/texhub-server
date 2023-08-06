use diesel::QueryDsl;
use log::error;
use rust_wheel::common::util::time_util::get_current_millisecond;
use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::doc::tex_doc_add::TexProjectAdd;
use crate::model::diesel::tex::custom_tex_models::TexProject;

pub fn get_prj_list(_tag: &String) -> Vec<TexProject>{
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    let cvs = query
        .load::<TexProject>(&mut get_connection());
    match cvs {
        Ok(result)=>{
            return result;
        },
        Err(err) =>{
            error!("get docs failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn create_doc(input_doc: &String) -> TexProject{
    let new_doc = TexProjectAdd{ 
        doc_name: input_doc.to_string(), 
        created_time: get_current_millisecond(), 
        updated_time: get_current_millisecond(), 
        user_id: 1, 
        doc_status: 1, 
        template_id: 1 
    };
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;

    let result = diesel::insert_into(tex_project)
            .values(&new_doc)
            .get_result::<TexProject>(&mut get_connection())
            .expect("get insert doc failed");
    return result;
}



