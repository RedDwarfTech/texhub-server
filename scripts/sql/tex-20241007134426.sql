--
-- PostgreSQL database dump
--

-- Dumped from database version 13.7
-- Dumped by pg_dump version 17.0

-- Started on 2024-10-07 13:44:26 CST

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 4 (class 2615 OID 2200)
-- Name: public; Type: SCHEMA; Schema: -; Owner: postgres
--

-- *not* creating schema, since initdb creates it


ALTER SCHEMA public OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 202 (class 1259 OID 4627397)
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.__diesel_schema_migrations OWNER TO postgres;

--
-- TOC entry 201 (class 1259 OID 4626912)
-- Name: tex_project; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_project (
    id bigint NOT NULL,
    proj_name character varying(256) NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    user_id bigint NOT NULL,
    proj_status integer DEFAULT 0 NOT NULL,
    template_id bigint NOT NULL,
    project_id character varying NOT NULL,
    nickname character varying NOT NULL,
    archive_status integer DEFAULT 0 NOT NULL,
    deleted integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.tex_project OWNER TO postgres;

--
-- TOC entry 3073 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.id IS '主键';


--
-- TOC entry 3074 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.proj_name; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.proj_name IS '项目名称';


--
-- TOC entry 3075 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.created_time IS '创建时间';


--
-- TOC entry 3076 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.updated_time IS '更新时间';


--
-- TOC entry 3077 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.user_id IS '用户ID';


--
-- TOC entry 3078 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.proj_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.proj_status IS '0:待生成 1:生成中 2:已生成';


--
-- TOC entry 3079 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.template_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.template_id IS '模版ID';


--
-- TOC entry 3080 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.project_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.project_id IS '项目ID';


--
-- TOC entry 3081 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.nickname; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.nickname IS '用户昵称';


--
-- TOC entry 3082 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.archive_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.archive_status IS '归档状态1: 已归档 0:未归档';


--
-- TOC entry 3083 (class 0 OID 0)
-- Dependencies: 201
-- Name: COLUMN tex_project.deleted; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_project.deleted IS '是否已删除 1:已删除 0:未删除';


--
-- TOC entry 200 (class 1259 OID 4626910)
-- Name: cv_main_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_project ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.cv_main_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 204 (class 1259 OID 4634084)
-- Name: tex_template; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_template (
    id bigint NOT NULL,
    name character varying(256) NOT NULL,
    remark character varying(256) NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    template_status integer DEFAULT 0 NOT NULL,
    template_id bigint NOT NULL,
    preview_url character varying,
    template_code character varying NOT NULL,
    online_status integer DEFAULT 1 NOT NULL,
    source character varying,
    font_size character varying,
    main_color character varying,
    theme character varying,
    language character varying NOT NULL,
    intro character varying NOT NULL,
    template_type integer NOT NULL,
    pdf_name character varying NOT NULL,
    main_file_name character varying NOT NULL
);


ALTER TABLE public.tex_template OWNER TO postgres;

--
-- TOC entry 3084 (class 0 OID 0)
-- Dependencies: 204
-- Name: TABLE tex_template; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON TABLE public.tex_template IS '模版';


--
-- TOC entry 3085 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.id IS '主键';


--
-- TOC entry 3086 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.name; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.name IS '简历模版名称';


--
-- TOC entry 3087 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.remark; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.remark IS '备注';


--
-- TOC entry 3088 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.created_time IS '创建时间';


--
-- TOC entry 3089 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.updated_time IS '更新时间';


--
-- TOC entry 3090 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.template_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.template_status IS '0:待生成 1:生成中 2:已生成';


--
-- TOC entry 3091 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.template_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.template_id IS '简历模版ID';


--
-- TOC entry 3092 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.preview_url; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.preview_url IS '模版预览地址';


--
-- TOC entry 3093 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.template_code; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.template_code IS '模版编码';


--
-- TOC entry 3094 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.online_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.online_status IS '在线状态';


--
-- TOC entry 3095 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.source; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.source IS '来源';


--
-- TOC entry 3096 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.font_size; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.font_size IS '字体大小';


--
-- TOC entry 3097 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.main_color; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.main_color IS '主色调';


--
-- TOC entry 3098 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.theme; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.theme IS '主题';


--
-- TOC entry 3099 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.language; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.language IS '语言';


--
-- TOC entry 3100 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.intro; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.intro IS '模版简介';


--
-- TOC entry 3101 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.template_type; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.template_type IS '模版类型1：简历 2、推荐信 3、Paper 4、毕业设计 5、书籍 6、其他';


--
-- TOC entry 3102 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.pdf_name; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.pdf_name IS 'pdf文件名';


--
-- TOC entry 3103 (class 0 OID 0)
-- Dependencies: 204
-- Name: COLUMN tex_template.main_file_name; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_template.main_file_name IS '入口文件名称';


--
-- TOC entry 203 (class 1259 OID 4634082)
-- Name: cv_template_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_template ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.cv_template_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 210 (class 1259 OID 4642587)
-- Name: tex_comp_queue; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_comp_queue (
    id bigint NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    user_id bigint NOT NULL,
    comp_status integer DEFAULT 0 NOT NULL,
    project_id character varying NOT NULL,
    version_no character varying NOT NULL,
    comp_result integer DEFAULT '-1'::integer NOT NULL,
    complete_time bigint DEFAULT 0 NOT NULL
);


ALTER TABLE public.tex_comp_queue OWNER TO postgres;

--
-- TOC entry 3104 (class 0 OID 0)
-- Dependencies: 210
-- Name: TABLE tex_comp_queue; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON TABLE public.tex_comp_queue IS '编译队列';


--
-- TOC entry 3105 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.id IS '主键';


--
-- TOC entry 3106 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.created_time IS '创建时间';


--
-- TOC entry 3107 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.updated_time IS '更新时间';


--
-- TOC entry 3108 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.user_id IS '用户ID';


--
-- TOC entry 3109 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.comp_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.comp_status IS '0:待生成 1:生成中 2:已生成';


--
-- TOC entry 3110 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.project_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.project_id IS '项目ID';


--
-- TOC entry 3111 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.version_no; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.version_no IS '版本号';


--
-- TOC entry 3112 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.comp_result; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.comp_result IS '编译结果 0:成功 1:失败 -1:未知';


--
-- TOC entry 3113 (class 0 OID 0)
-- Dependencies: 210
-- Name: COLUMN tex_comp_queue.complete_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_comp_queue.complete_time IS '完成时间';


--
-- TOC entry 209 (class 1259 OID 4642585)
-- Name: tex_comp_queue_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_comp_queue ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_comp_queue_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 220 (class 1259 OID 5434700)
-- Name: tex_user_config; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_user_config (
    id bigint NOT NULL,
    config_key character varying(256) NOT NULL,
    remark character varying(256) NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    config_value character varying NOT NULL,
    user_id bigint NOT NULL
);


ALTER TABLE public.tex_user_config OWNER TO postgres;

--
-- TOC entry 3114 (class 0 OID 0)
-- Dependencies: 220
-- Name: TABLE tex_user_config; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON TABLE public.tex_user_config IS '用户配置信息';


--
-- TOC entry 3115 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN tex_user_config.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_user_config.id IS '主键';


--
-- TOC entry 3116 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN tex_user_config.config_key; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_user_config.config_key IS '配置的Key';


--
-- TOC entry 3117 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN tex_user_config.remark; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_user_config.remark IS '备注';


--
-- TOC entry 3118 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN tex_user_config.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_user_config.created_time IS '创建时间';


--
-- TOC entry 3119 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN tex_user_config.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_user_config.updated_time IS '更新时间';


--
-- TOC entry 3120 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN tex_user_config.config_value; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_user_config.config_value IS '配置值';


--
-- TOC entry 3121 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN tex_user_config.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_user_config.user_id IS '用户ID';


--
-- TOC entry 219 (class 1259 OID 5434698)
-- Name: tex_config_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_user_config ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_config_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 206 (class 1259 OID 4640275)
-- Name: tex_file; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_file (
    id bigint NOT NULL,
    name character varying(256) NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    user_id bigint NOT NULL,
    doc_status integer DEFAULT 0 NOT NULL,
    project_id character varying NOT NULL,
    file_type integer NOT NULL,
    file_id character varying NOT NULL,
    parent character varying NOT NULL,
    main_flag smallint DEFAULT 0 NOT NULL,
    sort integer DEFAULT 0 NOT NULL,
    yjs_initial smallint DEFAULT 0 NOT NULL,
    file_path character varying DEFAULT '/'::character varying NOT NULL,
    CONSTRAINT tex_file_check CHECK ((length((project_id)::text) > 0))
);


ALTER TABLE public.tex_file OWNER TO postgres;

--
-- TOC entry 3122 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.id IS '主键';


--
-- TOC entry 3123 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.name; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.name IS '文档名称';


--
-- TOC entry 3124 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.created_time IS '创建时间';


--
-- TOC entry 3125 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.updated_time IS '更新时间';


--
-- TOC entry 3126 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.user_id IS '用户ID';


--
-- TOC entry 3127 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.doc_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.doc_status IS '文件状态';


--
-- TOC entry 3128 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.project_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.project_id IS '项目ID';


--
-- TOC entry 3129 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.file_type; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.file_type IS '0:目录 1.tex文件';


--
-- TOC entry 3130 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.file_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.file_id IS '文件ID';


--
-- TOC entry 3131 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.parent; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.parent IS '上级ID';


--
-- TOC entry 3132 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.main_flag; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.main_flag IS '是否是主文件';


--
-- TOC entry 3133 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.sort; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.sort IS '排序';


--
-- TOC entry 3134 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.yjs_initial; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.yjs_initial IS '是否已经在yjs中初始化';


--
-- TOC entry 3135 (class 0 OID 0)
-- Dependencies: 206
-- Name: COLUMN tex_file.file_path; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file.file_path IS '相对于本项目目录的路径';


--
-- TOC entry 205 (class 1259 OID 4640273)
-- Name: tex_doc_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_file ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_doc_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 212 (class 1259 OID 4830982)
-- Name: tex_file_version; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_file_version (
    id bigint NOT NULL,
    name character varying(256) NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    user_id bigint NOT NULL,
    project_id character varying NOT NULL,
    file_id character varying NOT NULL,
    content character varying NOT NULL,
    action integer NOT NULL,
    snapshot text NOT NULL,
    snapshot_hash character varying NOT NULL,
    version_status smallint DEFAULT 0 NOT NULL,
    CONSTRAINT tex_file_check_1 CHECK ((length((project_id)::text) > 0))
);


ALTER TABLE public.tex_file_version OWNER TO postgres;

--
-- TOC entry 3136 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.id IS '主键';


--
-- TOC entry 3137 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.name; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.name IS '文档名称';


--
-- TOC entry 3138 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.created_time IS '创建时间';


--
-- TOC entry 3139 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.updated_time IS '更新时间';


--
-- TOC entry 3140 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.user_id IS '用户ID';


--
-- TOC entry 3141 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.project_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.project_id IS '项目ID';


--
-- TOC entry 3142 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.file_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.file_id IS '文件ID';


--
-- TOC entry 3143 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.content; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.content IS '文件内容';


--
-- TOC entry 3144 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.action; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.action IS '1、编辑 2、重命名';


--
-- TOC entry 3145 (class 0 OID 0)
-- Dependencies: 212
-- Name: COLUMN tex_file_version.snapshot; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_file_version.snapshot IS '快照';


--
-- TOC entry 211 (class 1259 OID 4830980)
-- Name: tex_file_version_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_file_version ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_file_version_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 208 (class 1259 OID 4640297)
-- Name: tex_proj_editor; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_proj_editor (
    id bigint NOT NULL,
    role_id integer NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    user_id bigint NOT NULL,
    collar_status integer DEFAULT 0 NOT NULL,
    sort integer NOT NULL,
    project_id character varying NOT NULL,
    trash integer DEFAULT 0 NOT NULL,
    archive_status integer DEFAULT 0 NOT NULL,
    proj_status integer DEFAULT 1 NOT NULL,
    nickname character varying NOT NULL
);


ALTER TABLE public.tex_proj_editor OWNER TO postgres;

--
-- TOC entry 3146 (class 0 OID 0)
-- Dependencies: 208
-- Name: TABLE tex_proj_editor; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON TABLE public.tex_proj_editor IS '项目协作表';


--
-- TOC entry 3147 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.id IS '主键';


--
-- TOC entry 3148 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.role_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.role_id IS '角色1.owner 2.collarborator';


--
-- TOC entry 3149 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.created_time IS '创建时间';


--
-- TOC entry 3150 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.updated_time IS '更新时间';


--
-- TOC entry 3151 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.user_id IS '用户ID';


--
-- TOC entry 3152 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.collar_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.collar_status IS '协作状态 1:正常 2:退出协作';


--
-- TOC entry 3153 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.sort; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.sort IS '排序';


--
-- TOC entry 3154 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.project_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.project_id IS '项目ID';


--
-- TOC entry 3155 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.trash; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.trash IS '是否移入回收站 1.移入回收站 0.未移入回收站';


--
-- TOC entry 3156 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.archive_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.archive_status IS '归档状态1.已归档 0.未归档';


--
-- TOC entry 3157 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.proj_status; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.proj_status IS '1:正常 3:已归档 4: 已回收 5.已删除';


--
-- TOC entry 3158 (class 0 OID 0)
-- Dependencies: 208
-- Name: COLUMN tex_proj_editor.nickname; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_editor.nickname IS '用户昵称';


--
-- TOC entry 207 (class 1259 OID 4640295)
-- Name: tex_proj_editor_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_proj_editor ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_proj_editor_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 214 (class 1259 OID 5025356)
-- Name: tex_proj_folder; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_proj_folder (
    id bigint NOT NULL,
    folder_name character varying NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    user_id bigint NOT NULL,
    sort integer NOT NULL,
    proj_type integer DEFAULT 1 NOT NULL,
    default_folder integer DEFAULT 1 NOT NULL
);


ALTER TABLE public.tex_proj_folder OWNER TO postgres;

--
-- TOC entry 3159 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.id IS '主键';


--
-- TOC entry 3160 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.folder_name; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.folder_name IS '文件夹名称';


--
-- TOC entry 3161 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.created_time IS '创建时间';


--
-- TOC entry 3162 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.updated_time IS '更新时间';


--
-- TOC entry 3163 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.user_id IS '用户ID';


--
-- TOC entry 3164 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.sort; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.sort IS '排序';


--
-- TOC entry 3165 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.proj_type; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.proj_type IS '项目类型1.全部 2.分享给我 3.已归档 4.回收站';


--
-- TOC entry 3166 (class 0 OID 0)
-- Dependencies: 214
-- Name: COLUMN tex_proj_folder.default_folder; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder.default_folder IS '是否是默认文件夹';


--
-- TOC entry 216 (class 1259 OID 5035992)
-- Name: tex_proj_folder_map; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_proj_folder_map (
    id bigint NOT NULL,
    folder_id bigint NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    project_id character varying NOT NULL,
    user_id bigint NOT NULL,
    proj_type integer NOT NULL
);


ALTER TABLE public.tex_proj_folder_map OWNER TO postgres;

--
-- TOC entry 3167 (class 0 OID 0)
-- Dependencies: 216
-- Name: COLUMN tex_proj_folder_map.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder_map.id IS '主键';


--
-- TOC entry 3168 (class 0 OID 0)
-- Dependencies: 216
-- Name: COLUMN tex_proj_folder_map.folder_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder_map.folder_id IS '文件夹ID';


--
-- TOC entry 3169 (class 0 OID 0)
-- Dependencies: 216
-- Name: COLUMN tex_proj_folder_map.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder_map.created_time IS '创建时间';


--
-- TOC entry 3170 (class 0 OID 0)
-- Dependencies: 216
-- Name: COLUMN tex_proj_folder_map.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder_map.updated_time IS '更新时间';


--
-- TOC entry 3171 (class 0 OID 0)
-- Dependencies: 216
-- Name: COLUMN tex_proj_folder_map.project_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder_map.project_id IS '项目ID';


--
-- TOC entry 3172 (class 0 OID 0)
-- Dependencies: 216
-- Name: COLUMN tex_proj_folder_map.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder_map.user_id IS '用户ID';


--
-- TOC entry 3173 (class 0 OID 0)
-- Dependencies: 216
-- Name: COLUMN tex_proj_folder_map.proj_type; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_proj_folder_map.proj_type IS '项目类型1.全部 2.分享给我 3.已归档 4.回收站';


--
-- TOC entry 215 (class 1259 OID 5035990)
-- Name: tex_proj_folder_1_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_proj_folder_map ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_proj_folder_1_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 213 (class 1259 OID 5025354)
-- Name: tex_proj_folder_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_proj_folder ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_proj_folder_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 218 (class 1259 OID 5335027)
-- Name: tex_snippet; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tex_snippet (
    id bigint NOT NULL,
    snippet character varying NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    user_id bigint NOT NULL,
    sort integer NOT NULL,
    title character varying NOT NULL
);


ALTER TABLE public.tex_snippet OWNER TO postgres;

--
-- TOC entry 3174 (class 0 OID 0)
-- Dependencies: 218
-- Name: COLUMN tex_snippet.id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_snippet.id IS '主键';


--
-- TOC entry 3175 (class 0 OID 0)
-- Dependencies: 218
-- Name: COLUMN tex_snippet.snippet; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_snippet.snippet IS '文件夹名称';


--
-- TOC entry 3176 (class 0 OID 0)
-- Dependencies: 218
-- Name: COLUMN tex_snippet.created_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_snippet.created_time IS '创建时间';


--
-- TOC entry 3177 (class 0 OID 0)
-- Dependencies: 218
-- Name: COLUMN tex_snippet.updated_time; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_snippet.updated_time IS '更新时间';


--
-- TOC entry 3178 (class 0 OID 0)
-- Dependencies: 218
-- Name: COLUMN tex_snippet.user_id; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_snippet.user_id IS '用户ID';


--
-- TOC entry 3179 (class 0 OID 0)
-- Dependencies: 218
-- Name: COLUMN tex_snippet.sort; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.tex_snippet.sort IS '排序';


--
-- TOC entry 217 (class 1259 OID 5335025)
-- Name: tex_snippet_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.tex_snippet ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_snippet_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 2901 (class 2606 OID 4627402)
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- TOC entry 2903 (class 2606 OID 4634093)
-- Name: tex_template cv_template_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_template
    ADD CONSTRAINT cv_template_pk PRIMARY KEY (id);


--
-- TOC entry 2934 (class 2606 OID 5434709)
-- Name: tex_user_config cv_template_pk_1; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_user_config
    ADD CONSTRAINT cv_template_pk_1 PRIMARY KEY (id);


--
-- TOC entry 2905 (class 2606 OID 4634095)
-- Name: tex_template cv_template_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_template
    ADD CONSTRAINT cv_template_un UNIQUE (template_code);


--
-- TOC entry 2917 (class 2606 OID 4642595)
-- Name: tex_comp_queue tex_comp_queue_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_comp_queue
    ADD CONSTRAINT tex_comp_queue_pk PRIMARY KEY (id);


--
-- TOC entry 2909 (class 2606 OID 4640283)
-- Name: tex_file tex_doc_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_file
    ADD CONSTRAINT tex_doc_pk PRIMARY KEY (id);


--
-- TOC entry 2919 (class 2606 OID 4830995)
-- Name: tex_file_version tex_doc_pk_1; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_file_version
    ADD CONSTRAINT tex_doc_pk_1 PRIMARY KEY (id);


--
-- TOC entry 2911 (class 2606 OID 4640333)
-- Name: tex_file tex_file_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_file
    ADD CONSTRAINT tex_file_un UNIQUE (parent, name, file_type);


--
-- TOC entry 2913 (class 2606 OID 4641939)
-- Name: tex_proj_editor tex_proj_editor_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_proj_editor
    ADD CONSTRAINT tex_proj_editor_un UNIQUE (role_id, user_id, project_id);


--
-- TOC entry 2926 (class 2606 OID 5036424)
-- Name: tex_proj_folder_map tex_proj_folder_map_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_proj_folder_map
    ADD CONSTRAINT tex_proj_folder_map_un UNIQUE (folder_id, project_id);


--
-- TOC entry 2928 (class 2606 OID 5043782)
-- Name: tex_proj_folder_map tex_proj_folder_map_user_proj_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_proj_folder_map
    ADD CONSTRAINT tex_proj_folder_map_user_proj_un UNIQUE (user_id, project_id);


--
-- TOC entry 2922 (class 2606 OID 5026643)
-- Name: tex_proj_folder tex_proj_folder_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_proj_folder
    ADD CONSTRAINT tex_proj_folder_un UNIQUE (folder_name, user_id, proj_type);


--
-- TOC entry 2915 (class 2606 OID 4640305)
-- Name: tex_proj_editor tex_project_editor_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_proj_editor
    ADD CONSTRAINT tex_project_editor_pk PRIMARY KEY (id);


--
-- TOC entry 2930 (class 2606 OID 5036000)
-- Name: tex_proj_folder_map tex_project_folder_map_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_proj_folder_map
    ADD CONSTRAINT tex_project_folder_map_pk PRIMARY KEY (id);


--
-- TOC entry 2924 (class 2606 OID 5025366)
-- Name: tex_proj_folder tex_project_folder_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_proj_folder
    ADD CONSTRAINT tex_project_folder_pk PRIMARY KEY (id);


--
-- TOC entry 2932 (class 2606 OID 5335036)
-- Name: tex_snippet tex_project_folder_pk_1; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_snippet
    ADD CONSTRAINT tex_project_folder_pk_1 PRIMARY KEY (id);


--
-- TOC entry 2897 (class 2606 OID 4626920)
-- Name: tex_project tex_project_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_project
    ADD CONSTRAINT tex_project_pk PRIMARY KEY (id);


--
-- TOC entry 2899 (class 2606 OID 4640290)
-- Name: tex_project tex_project_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_project
    ADD CONSTRAINT tex_project_un UNIQUE (project_id);


--
-- TOC entry 2907 (class 2606 OID 4676900)
-- Name: tex_template tex_template_id_un; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_template
    ADD CONSTRAINT tex_template_id_un UNIQUE (template_id);


--
-- TOC entry 2936 (class 2606 OID 5434715)
-- Name: tex_user_config tex_user_config_unique; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tex_user_config
    ADD CONSTRAINT tex_user_config_unique UNIQUE (config_key, user_id);


--
-- TOC entry 2920 (class 1259 OID 5431736)
-- Name: tex_file_version_file_id_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX tex_file_version_file_id_idx ON public.tex_file_version USING btree (file_id, snapshot_hash);


--
-- TOC entry 3072 (class 0 OID 0)
-- Dependencies: 4
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
--

REVOKE USAGE ON SCHEMA public FROM PUBLIC;
GRANT ALL ON SCHEMA public TO PUBLIC;


-- Completed on 2024-10-07 13:44:31 CST

--
-- PostgreSQL database dump complete
--

