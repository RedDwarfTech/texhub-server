table! {
    tex_file (id) {
        id -> Int8,
        name -> Varchar,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        doc_status -> Int4,
        project_id -> Varchar,
        file_type -> Int4,
        file_id -> Varchar,
        parent -> Varchar,
        main_flag -> Int2,
        sort -> Int4,
        yjs_initial -> Int2,
        file_path -> Varchar
    }
}

table! {
    tex_proj_editor (id) {
        id -> Int8,
        role_id -> Int4,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        collar_status -> Int4,
        sort -> Int4,
        project_id -> Varchar,
    }
}

table! {
    tex_project (id) {
        id -> Int8,
        proj_name -> Varchar,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        proj_status -> Int4,
        template_id -> Int8,
        project_id -> Varchar,
        nickname -> Varchar,
    }
}

table! {
    tex_template (id) {
        id -> Int8,
        name -> Varchar,
        remark -> Varchar,
        created_time -> Int8,
        updated_time -> Int8,
        template_status -> Int4,
        template_id -> Int8,
        preview_url -> Nullable<Varchar>,
        template_code -> Varchar,
        online_status -> Int4,
        source -> Nullable<Varchar>,
        font_size -> Nullable<Varchar>,
        main_color -> Nullable<Varchar>,
        theme -> Nullable<Varchar>,
        language -> Varchar,
        intro -> Varchar,
        template_type -> Int4,
        pdf_name -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    tex_file,
    tex_proj_editor,
    tex_project,
    tex_template,
);
