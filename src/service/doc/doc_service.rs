use diesel::QueryDsl;
use log::error;
use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::tex::custom_tex_models::TexDoc;

pub fn get_doc_list(_tag: &String) -> Vec<TexDoc>{
    use crate::model::diesel::tex::tex_schema::tex_doc as cv_work_table;
    let query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    let cvs = query
        .load::<TexDoc>(&mut get_connection());
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






