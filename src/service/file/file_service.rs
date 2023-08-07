use diesel::{ExpressionMethods, QueryDsl, sql_query};
use log::error;
use rust_wheel::common::util::convert_to_tree_generic::convert_to_tree;
use rust_wheel::common::util::model_convert::map_entity;
use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::model::response::file::file_tree_resp::FileTreeResp;

pub fn get_file_list(parent_id: &String) -> Vec<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::parent.eq(parent_id));
    let cvs = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_file_tree(parent_id: &String) -> Vec<FileTreeResp> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::parent.eq(parent_id));
    let cvs = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return find_sub_menu_cte_impl(&result,parent_id);
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn find_sub_menu_cte_impl(_root_menus: &Vec<TexFile>, root_id: &String) -> Vec<FileTreeResp>{
    let mut connection = get_connection();
    let cte_query_sub_menus = format!(" with recursive sub_files as (
        select 
          id, 
          name, 
          file_id, 
          sort 
        from 
          tex_file mr 
        where 
          parent = '{}' 
        union all 
        select 
          origin.id, 
          origin.name, 
          origin.file_id, 
          origin.sort 
        from 
          sub_files 
          join tex_file origin on origin.parent = sub_files.file_id
      ) 
      select 
        id, 
        name, 
        file_id, 
        sort 
      from 
        sub_files 
      order by 
        sort asc;      
    ",root_id);
    let cte_menus = sql_query(cte_query_sub_menus)
        .load::<TexFile>(&mut connection)
        .expect("Error find file");
    let menu_resource_resp:Vec<FileTreeResp> = map_entity(cte_menus);
    return convert_to_tree_impl(&menu_resource_resp, root_id);
}

fn convert_to_tree_impl(contents: &Vec<FileTreeResp>, root_id: &str) -> Vec<FileTreeResp> {
    let root_element: Vec<_> = contents.iter()
        .filter(|content| content.parent == root_id)
        .collect();
    let sub_element: Vec<_> = contents.iter()
        .filter(|content| content.parent != root_id)
        .collect();
    let result = convert_to_tree(&root_element, &sub_element);
    return result;
}