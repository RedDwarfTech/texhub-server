table! {
    tex_doc (id) {
        id -> Int8,
        doc_name -> Varchar,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        doc_status -> Int4,
        template_id -> Int8,
    }
}

table! {
    tex_template (id) {
        id -> Int8,
        cv_name -> Varchar,
        created_time -> Int8,
        updated_time -> Int8,
        user_id -> Int8,
        cv_status -> Int4,
        template_id -> Int8,
        employee_name -> Nullable<Varchar>,
        birthday -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        job -> Nullable<Varchar>,
        workplace -> Nullable<Varchar>,
        stackoverflow -> Nullable<Varchar>,
        github -> Nullable<Varchar>,
        blog -> Nullable<Varchar>,
        item_order -> Varchar,
        remark -> Nullable<Varchar>,
        main_color -> Nullable<Varchar>,
        theme -> Nullable<Varchar>,
        font_size -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    tex_doc,
    tex_template,
);
