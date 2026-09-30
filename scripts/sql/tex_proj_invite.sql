--
-- 项目邀请凭证
--
-- 设计要点：
-- 1. 每个项目至多一条 active=1 的记录（部分唯一索引），实现「单活跃 token」：
--    重复点「生成邀请链接」复用同一条，链接稳定不变；「撤销」置 active=0，
--    之后可重新签发。
-- 2. token_cipher 存 AES-256-GCM 密文而非明文，也不是裸 hash。裸 hash 无法让
--    Owner 二次查看同一条链接（只能靠不断轮换，反而破坏了链接稳定性）；
--    明文则一旦 DB 泄露所有邀请码即刻可用。密文把两者兼顾：Owner 可复现链接，
--    但 DB dump 本身不足以完成入伙。
-- 3. token_hash 只是等值检索的索引加速项，不是秘密：真正的凭证保护来自
--    token_cipher 的解密校验。有了它，校验一次邀请码是 O(1) 索引命中，
--    而不必遍历所有 active 行逐个做 AEAD 解密（那会让每次请求的耗时随
--    邀请码总数线性增长，是个很容易被拿来打 DoS 的放大点）。
-- 4. project_id 是密文的 AAD，攻击者无法把 A 项目邀请码搬到 B 项目上使用。
--

CREATE TABLE public.tex_proj_invite (
    id bigint NOT NULL,
    created_time bigint NOT NULL,
    updated_time bigint NOT NULL,
    project_id character varying NOT NULL,
    token_hash character varying NOT NULL,
    token_cipher bytea NOT NULL,
    role_id integer DEFAULT 2 NOT NULL,
    expire_at bigint DEFAULT 0 NOT NULL,
    max_uses integer DEFAULT 0 NOT NULL,
    used_count integer DEFAULT 0 NOT NULL,
    created_by bigint NOT NULL,
    active smallint DEFAULT 1 NOT NULL
);


ALTER TABLE public.tex_proj_invite OWNER TO postgres;

COMMENT ON TABLE public.tex_proj_invite IS '项目邀请凭证';

COMMENT ON COLUMN public.tex_proj_invite.id IS '主键';

COMMENT ON COLUMN public.tex_proj_invite.created_time IS '创建时间';

COMMENT ON COLUMN public.tex_proj_invite.updated_time IS '更新时间';

COMMENT ON COLUMN public.tex_proj_invite.project_id IS '项目ID';

COMMENT ON COLUMN public.tex_proj_invite.token_hash IS '邀请码SHA256十六进制，仅用于等值检索';

COMMENT ON COLUMN public.tex_proj_invite.token_cipher IS '邀请码密文(AES-256-GCM)，附加认证数据为project_id';

COMMENT ON COLUMN public.tex_proj_invite.role_id IS '通过邀请获得的角色 2:协作者';

COMMENT ON COLUMN public.tex_proj_invite.expire_at IS '过期时间时(毫秒)，0为永不过期';

COMMENT ON COLUMN public.tex_proj_invite.max_uses IS '最大使用次数，0为不限次';

COMMENT ON COLUMN public.tex_proj_invite.used_count IS '已使用次数';

COMMENT ON COLUMN public.tex_proj_invite.created_by IS '签发人用户ID(项目所有者)';

COMMENT ON COLUMN public.tex_proj_invite.active IS '是否有效 1:有效 0:已撤销';


ALTER TABLE public.tex_proj_invite ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.tex_proj_invite_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


ALTER TABLE ONLY public.tex_proj_invite
    ADD CONSTRAINT tex_proj_invite_pkey PRIMARY KEY (id);


-- 单活跃凭证：同一项目只允许一条 active=1
CREATE UNIQUE INDEX tex_proj_invite_active_un ON public.tex_proj_invite (project_id) WHERE active = 1;


-- 校验凭证时按 token_hash 唯一命中
CREATE UNIQUE INDEX tex_proj_invite_hash_un ON public.tex_proj_invite (token_hash);


-- 当前凭证查询走上面的部分唯一索引；这里是为「列出项目全部历史凭证（含已撤销）」
-- 预留的索引。active_un 只覆盖 active=1 的行，查历史时用不上它。
CREATE INDEX tex_proj_invite_proj_idx ON public.tex_proj_invite (project_id);
