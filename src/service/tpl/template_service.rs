use diesel::{QueryDsl, ExpressionMethods};
use log::error;
use rust_wheel::common::util::time_util::get_current_millisecond;
use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::doc::tex_doc_add::TexDocAdd;
use crate::model::diesel::tex::custom_tex_models::{TexDoc, TexTemplate};

pub fn get_tpl_list(_tag: &String) -> Vec<TexTemplate>{
    use crate::model::diesel::tex::tex_schema::tex_template as cv_tpl_table;
    let mut query = cv_tpl_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_tpl_table::online_status.eq(1));
    let cvs = query
        .load::<TexTemplate>(&mut get_connection());
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


pub fn create_tpl(input_doc: &String) -> TexDoc{
    let new_doc = TexDocAdd{ 
        doc_name: input_doc.to_string(), 
        created_time: get_current_millisecond(), 
        updated_time: get_current_millisecond(), 
        user_id: 1, 
        doc_status: 1, 
        template_id: 1 
    };
    use crate::model::diesel::tex::tex_schema::tex_doc::dsl::*;

    let result = diesel::insert_into(tex_doc)
            .values(&new_doc)
            .get_result::<TexDoc>(&mut get_connection())
            .expect("get insert doc failed");
    return result;
}



