use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::doc::tex_doc_add::TexProjectAdd;
use crate::model::diesel::tex::custom_tex_models::TexProject;
use crate::{common::database::get_connection, model::diesel::tex::custom_tex_models::TexFile};
use diesel::{sql_query, Connection, ExpressionMethods, PgConnection, QueryDsl};
use log::{error, warn};
use rust_wheel::common::util::time_util::get_current_millisecond;

pub fn get_prj_list(_tag: &String) -> Vec<TexProject> {
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    let cvs = query.load::<TexProject>(&mut get_connection());
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

pub fn create_project(input_doc: &String) -> TexProject {
    let new_doc = TexProjectAdd {
        doc_name: input_doc.to_string(),
        created_time: get_current_millisecond(),
        updated_time: get_current_millisecond(),
        user_id: 1,
        doc_status: 1,
        template_id: 1,
    };
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let result = diesel::insert_into(tex_project)
        .values(&new_doc)
        .get_result::<TexProject>(&mut get_connection())
        .expect("get insert doc failed");
    return result;
}

pub fn del_project(del_project_id: &String) {
    let mut connection = get_connection();
    let result = connection.transaction(|connection| {
        let delete_result = del_project_impl(del_project_id, connection);
        match delete_result {
            Ok(rows) => {
                if rows == 0 {
                    warn!("the delete project effect {} rows, project id: {}", rows, del_project_id);
                }
                del_project_file(del_project_id, connection);
                Ok("")
            }
            Err(e) => diesel::result::QueryResult::Err(e),
        }
    });
    match result {
        Ok(_) => {}
        Err(e) => {
            error!(
                "transaction failed, project id: {},error:{}",
                del_project_id, e
            );
        }
    }
}

pub fn del_project_impl(
    del_project_id: &String,
    connection: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let predicate =
        crate::model::diesel::tex::tex_schema::tex_project::project_id.eq(del_project_id);
    let delete_result = diesel::delete(tex_project.filter(predicate)).execute(connection);
    return delete_result;
}

pub fn del_project_file(del_project_id: &String, connection: &mut PgConnection) {
    let del_command = format!(
        "WITH RECURSIVE x AS (
        SELECT id
        FROM   tex_file
        WHERE parent = {}
     
        UNION  ALL
        SELECT id
        FROM   x
        JOIN   tex_file a ON a.parent = x.id
        )
     DELETE FROM tex_file a
     USING  x
     WHERE a.id = x.id",
        del_project_id
    );
    let cte_menus = sql_query(&del_command).load::<TexFile>(connection);
    match cte_menus {
        Ok(_) => {}
        Err(_) => {
            error!(
                "delete project file failed, project id: {}, command:{}",
                del_project_id, del_command
            );
        }
    }
}
