table! {
    tex_comp_queue (id) {
        id -> Int8,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        comp_status -> Int4,
        project_id -> Varchar,
        version_no -> Varchar,
        comp_result -> Int4,
        complete_time -> Int8,
    }
}

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
        file_path -> Varchar,
    }
}

table! {
    tex_file_version (id) {
        id -> Int8,
        name -> Varchar,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        project_id -> Varchar,
        file_id -> Varchar,
        content -> Varchar,
        action -> Int4,
        snapshot -> Text,
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
        trash -> Int4,
        archive_status -> Int4,
        proj_status -> Int4,
    }
}

table! {
    tex_proj_folder (id) {
        id -> Int8,
        folder_name -> Varchar,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        sort -> Int4,
        proj_type -> Int4,
        default_folder -> Int4,
    }
}

table! {
    tex_proj_folder_map (id) {
        id -> Int8,
        folder_id -> Int8,
        created_time -> Int8,
        updated_time -> Int8,
        project_id -> Varchar,
        user_id -> Int8,
        proj_type -> Int4,
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
        archive_status -> Int4,
        deleted -> Int4,
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
        main_file_name -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    tex_comp_queue,
    tex_file,
    tex_file_version,
    tex_proj_editor,
    tex_proj_folder,
    tex_proj_folder_map,
    tex_project,
    tex_template,
);
