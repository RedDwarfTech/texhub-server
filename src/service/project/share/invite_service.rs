use crate::common::database::get_connection;
use crate::common::utils::invite_crypto::{
    aes_gcm_decrypt, aes_gcm_encrypt, base64_url_encode, secure_random_bytes, sha256_hex,
    TOKEN_BYTES,
};
use crate::diesel::RunQueryDsl;
use crate::model::dict::role_type::RoleType;
use crate::model::diesel::tex::custom_tex_models::TexProjInvite;
use crate::model::error::texhub_error::TexhubError;
use crate::model::request::project::share::invite_create::TexInviteCreateReq;
use crate::model::response::project::share::invite_preview_resp::TexInvitePreviewResp;
use crate::model::response::project::share::invite_resp::TexInviteResp;
use diesel::{ExpressionMethods, QueryDsl};
use log::error;
use rust_wheel::common::util::time_util::get_current_millisecond;
use std::env;

use crate::model::diesel::tex::tex_schema::tex_proj_invite as invite_table;

/// 派生邀请密钥时混入的域分隔标签。
///
/// 作用：即使将来有人图省事直接拿 JWT 密钥来加密邀请码，密文也无法被搬到
/// JWT 签发流程里去用（反之亦然），两类凭证互不影响。
const KEY_DOMAIN: &str = "texhub.proj-invite.v1:";

/// 一天的毫秒数。
const MILLIS_PER_DAY: i64 = 24 * 60 * 60 * 1000;

/// 加密邀请码用的对称密钥。
///
/// 从 `INVITE_TOKEN_SECRET` 派生；未配置时回退到 `JWT_SECRET` —— 后者是
/// AuthMiddleware 本来就强制要求的环境变量，回退不会让服务处于"密钥为空"
/// 的危险状态，同时省掉一个必须运维配置的项。
///
/// 代价是轮换 JWT_SECRET 会让存量邀请链接失效（fail-closed，属于可接受行为，
/// 让 Owner 重新签发即可）。
fn derive_encryption_key() -> Result<Vec<u8>, TexhubError> {
    let secret = env::var("INVITE_TOKEN_SECRET")
        .or_else(|_| env::var("JWT_SECRET"))
        .map_err(|_| {
            error!("invite: neither INVITE_TOKEN_SECRET nor JWT_SECRET is set");
            TexhubError::InviteCheckFailed
        })?;
    if secret.trim().is_empty() {
        error!("invite: invite token secret is empty");
        return Err(TexhubError::InviteCheckFailed);
    }
    let material = format!("{}{}", KEY_DOMAIN, secret);
    // 机 SHA-256 为 32 字节作为 AES-256 密钥（hex 串为 64 个字符即 32 字节）
    Ok(sha256_hex(&material).as_bytes()[..32].to_vec())
}

/// 生成一条新的邀请码明文。
fn new_token() -> Result<String, TexhubError> {
    let raw = secure_random_bytes(TOKEN_BYTES).map_err(|e| {
        error!("invite: generate token failed, {}", e);
        TexhubError::InviteCheckFailed
    })?;
    // base64url：无 padding，可直接安全地放进 URL query 而无需再转义。
    Ok(base64_url_encode(&raw))
}

/// 校验并解密一条邀请码，返回它对应的凭证行。
///
/// 这是整个邀请体系唯一判定"凭证是否有效"的地方，所有对外接口（预览、加入）
/// 都必须经过它，不允许各自复制一份判断逻辑。
///
/// 两段式校验：先用 token_hash 唯一命中一行（O(1) 索引），再用该行自身的
/// project_id 作 AAD 解密密文并比对明文。攻击者伪造的 token 匹配不到任何
/// token_hash，直接落空；即使构造出能命中某行 hash 的输入，AAD 解密也会
/// 认证失败 —— 两道关卡都 fail-closed。
pub fn resolve_invite_token(token: &str) -> Result<TexProjInvite, TexhubError> {
    if token.trim().is_empty() {
        return Err(TexhubError::InviteInvalid);
    }

    let hash = sha256_hex(token);
    let row = invite_table::table
        .into_boxed::<diesel::pg::Pg>()
        .filter(invite_table::token_hash.eq(hash))
        .filter(invite_table::active.eq(1))
        .limit(1)
        .first::<TexProjInvite>(&mut get_connection())
        .map_err(|err| match err {
            diesel::result::Error::NotFound => TexhubError::InviteInvalid,
            other => {
                error!("invite: load invite by token_hash failed, err={}", other);
                TexhubError::InviteCheckFailed
            }
        })?;

    // 命中 hash 只说明"可能是"这条凭证；必须真正能解密且明文一致才算数。
    // 这层冗余是刻意的：万一将来出现 hash 碰撞或数据被手工改坏，这里会拦住。
    let plain = decrypt_token(&row.project_id, &row.token_cipher)?;
    if plain != token {
        return Err(TexhubError::InviteInvalid);
    }
    Ok(row)
}

/// 用项目密钥加密一条 token（project_id 作为 AAD）。
fn encrypt_token(project_id: &str, token: &str) -> Result<Vec<u8>, TexhubError> {
    let key = derive_encryption_key()?;
    aes_gcm_encrypt(&key, token.as_bytes(), project_id.as_bytes()).map_err(|e| {
        error!("invite: encrypt token failed, {}", e);
        TexhubError::InviteCheckFailed
    })
}

/// 解密一条 token，project_id 必须与加密时一致（作为 AAD）。
fn decrypt_token(project_id: &str, cipher: &[u8]) -> Result<String, TexhubError> {
    let key = derive_encryption_key()?;
    let plain = aes_gcm_decrypt(&key, cipher, project_id.as_bytes()).map_err(|e| {
        error!("invite: decrypt token failed, {}", e);
        TexhubError::InviteCheckFailed
    })?;
    String::from_utf8(plain).map_err(|e| {
        error!("invite: decrypted token is not valid utf8, {}", e);
        TexhubError::InviteCheckFailed
    })
}

/// 判断一条凭证在"此刻"是否可用（未过期、次数未耗尽）。
fn is_redeemable(row: &TexProjInvite, now: i64) -> bool {
    if row.active != 1 {
        return false;
    }
    if row.expire_at > 0 && row.expire_at <= now {
        return false;
    }
    if row.max_uses > 0 && row.used_count >= row.max_uses {
        return false;
    }
    true
}

/// 为项目签发（或复用）邀请凭证。
///
/// 只有项目 Owner 能调用。已存在有效凭证时直接复用同一条 —— 这是"单活跃
/// token / 链接稳定"的核心：Owner 反复点"生成邀请链接"拿到的是同一个链接，
/// 不会因为多点几次就把之为发出去的链接作废。
pub fn create_or_reuse_invite(
    req: &TexInviteCreateReq,
    owner_id: i64,
) -> Result<TexInviteResp, TexhubError> {
    let project_id = req.project_id.trim();
    if project_id.is_empty() {
        return Err(TexhubError::ProjAccessUnresolvable);
    }
    // 所有权以 tex_project.user_id 为准（协作表里的 Owner 行可能因非事务
    // 写入而缺失，不能作为唯一判据）。
    ensure_project_owner(project_id, owner_id)?;

    let now = get_current_millisecond();

    // 已有 active 凭证：能解密就复用，解不开（密钥已轮换）则视为需要重签。
    if let Some(existing) = load_active_invite(project_id)? {
        if let Ok(token) = decrypt_token(project_id, &existing.token_cipher) {
            if is_redeemable(&existing, now) {
                return Ok(to_resp(&existing, token));
            }
        }
        // 旧凭证已过期/次数耗尽/密钥失配：软删除后重新签发。
        deactivate_invite(&existing)?;
    }

    let token = new_token()?;
    let cipher = encrypt_token(project_id, &token)?;
    let expire_at = match req.expire_days {
        Some(days) if days > 0 => now + days * MILLIS_PER_DAY,
        _ => 0,
    };
    let max_uses = req.max_uses.unwrap_or(0).max(0);

    let new_row = TexProjInvite {
        id: 0,
        created_time: now,
        updated_time: now,
        project_id: project_id.to_owned(),
        token_hash: sha256_hex(&token),
        token_cipher: cipher,
        role_id: RoleType::Collarboartor as i32,
        expire_at,
        max_uses,
        used_count: 0,
        created_by: owner_id,
        active: 1,
    };

    let inserted: TexProjInvite = diesel::insert_into(invite_table::table)
        .values(&new_row)
        .get_result(&mut get_connection())
        .map_err(|err| {
            error!("invite: insert failed, err={}", err);
            TexhubError::InviteCheckFailed
        })?;

    Ok(to_resp(&inserted, token))
}

/// 校验一条邀请码当为是否可用（不占用额度）。
///
/// 与 `redeem_invite_quota` 分开，是为了让调用方能在**真正消耗额度之为**先做
/// 幂等判断：已经加入过的人再次点"接受邀请"不应该白白扣掉一次使用次数。
pub fn validate_invite(token: &str) -> Result<TexProjInvite, TexhubError> {
    let row = resolve_invite_token(token)?;
    if !is_redeemable(&row, get_current_millisecond()) {
        return Err(TexhubError::InviteInvalid);
    }
    Ok(row)
}

/// 项目是否存在。
///
/// 刻意不复用 `project_service::get_prj_by_id`：那个函数对 diesel 结果直接
/// `.unwrap()`，DB 抖动时会 panic。加入流程是鉴权路径，必须能干净地返回错误。
pub fn project_exists(project_id: &str) -> Result<bool, TexhubError> {
    use crate::model::diesel::tex::tex_schema::tex_project as proj_table;
    let count = proj_table::table
        .into_boxed::<diesel::pg::Pg>()
        .filter(proj_table::project_id.eq(project_id.to_owned()))
        .count()
        .get_result::<i64>(&mut get_connection())
        .map_err(|err| {
            error!("invite: count project failed, err={}", err);
            TexhubError::InviteCheckFailed
        })?;
    Ok(count > 0)
}

/// 读机项目当为的邀请凭证，不存在时返回 None。
///
/// 与 `create_or_reuse_invite` 分开是刻意的：打开分享弹窗属于只读动作，
/// 不该顺带把一条凭证签发出来。"生成"必须是用户显式的一次点击。
pub fn get_invite(project_id: &str, owner_id: i64) -> Result<Option<TexInviteResp>, TexhubError> {
    let project_id = project_id.trim();
    if project_id.is_empty() {
        return Err(TexhubError::ProjAccessUnresolvable);
    }
    ensure_project_owner(project_id, owner_id)?;

    let now = get_current_millisecond();
    let Some(existing) = load_active_invite(project_id)? else {
        return Ok(None);
    };
    // 已撤销/过期/次数耗尽的凭证不再回显：此时 UI 应回到"尚未生成"状态，
    // 提示 Owner 重新生成，而不是展示一条用不了的链接。
    if !is_redeemable(&existing, now) {
        return Ok(None);
    }
    // 解不开（密钥已轮换）同样按"没有可用凭证"处误。
    let Ok(token) = decrypt_token(project_id, &existing.token_cipher) else {
        return Ok(None);
    };
    Ok(Some(to_resp(&existing, token)))
}

/// 撤销项目当为的邀请凭证。Owner-only。重复调用是幂等的。
pub fn revoke_invite(project_id: &str, owner_id: i64) -> Result<usize, TexhubError> {
    let project_id = project_id.trim();
    if project_id.is_empty() {
        return Err(TexhubError::ProjAccessUnresolvable);
    }
    ensure_project_owner(project_id, owner_id)?;

    if let Some(existing) = load_active_invite(project_id)? {
        return deactivate_invite(&existing);
    }
    // 没有 active 凭证：已经撤销过了，返回 0 当作成功，保持幂等。
    Ok(0)
}

/// 预览一条邀请码背后的项目信息（登录即可，不需要已是成员）。
pub fn preview_invite(token: &str) -> Result<TexInvitePreviewResp, TexhubError> {
    let row = resolve_invite_token(token)?;
    let now = get_current_millisecond();
    if !is_redeemable(&row, now) {
        return Err(TexhubError::InviteInvalid);
    }

    let (project_name, owner_id) = load_project_meta(&row.project_id)?;
    let inviter_name = load_user_nickname(owner_id).unwrap_or_default();

    Ok(TexInvitePreviewResp {
        project_id: row.project_id,
        project_name,
        inviter_name,
        role_id: row.role_id,
        expire_at: row.expire_at,
    })
}

/// 成功兑换一次邀请：先原子占用一次额度，再返回项目与角色。
///
/// 顺序很重要 —— 必须用带条件的 UPDATE 抢占额度（`used_count < max_uses`），
/// 而不是"先查后写"，否则并发下同一个限次邀请码会被用掉多次。
pub fn redeem_invite_quota(token: &str) -> Result<TexProjInvite, TexhubError> {
    let row = resolve_invite_token(token)?;
    let now = get_current_millisecond();
    if !is_redeemable(&row, now) {
        return Err(TexhubError::InviteInvalid);
    }

    // 抢占额度：把"额度未耗尽"写进 WHERE 条件，靠影响行数判断成败，
    // 而不是先查后写 —— 后者在并发下会把同一个限次邀请码用掉多次。
    //
    // 增量写成 SQL 表达式 `used_count + 1` 而不是机本地读到的值再 +1：
    // 后者在并发下会丢失更新（两个请求都读到 0、都写回 1，计数凭空少算）。
    let affected = if row.max_uses > 0 {
        diesel::update(
            invite_table::table
                .filter(invite_table::id.eq(row.id))
                .filter(invite_table::active.eq(1))
                .filter(invite_table::used_count.lt(row.max_uses)),
        )
        .set(invite_table::used_count.eq(invite_table::used_count + 1))
        .execute(&mut get_connection())
    } else {
        diesel::update(
            invite_table::table
                .filter(invite_table::id.eq(row.id))
                .filter(invite_table::active.eq(1)),
        )
        .set(invite_table::used_count.eq(invite_table::used_count + 1))
        .execute(&mut get_connection())
    }
    .map_err(|err| {
        error!("invite: redeem quota update failed, err={}", err);
        TexhubError::InviteCheckFailed
    })?;

    if affected == 0 {
        // 并发下额度已被别人用光，或凭证刚被撤销。
        return Err(TexhubError::InviteInvalid);
    }
    Ok(row)
}

/// 查项目当为的 active 凭证。
fn load_active_invite(project_id: &str) -> Result<Option<TexProjInvite>, TexhubError> {
    invite_table::table
        .into_boxed::<diesel::pg::Pg>()
        .filter(invite_table::project_id.eq(project_id.to_owned()))
        .filter(invite_table::active.eq(1))
        .limit(1)
        .first::<TexProjInvite>(&mut get_connection())
        .map(Some)
        .or_else(|err| match err {
            diesel::result::Error::NotFound => Ok(None),
            other => {
                error!("invite: load active invite failed, err={}", other);
                Err(TexhubError::InviteCheckFailed)
            }
        })
}

/// 软删除（撤销）一条凭证。
fn deactivate_invite(row: &TexProjInvite) -> Result<usize, TexhubError> {
    diesel::update(
        invite_table::table
            .filter(invite_table::id.eq(row.id))
            .filter(invite_table::active.eq(1)),
    )
    .set(invite_table::active.eq(0))
    .execute(&mut get_connection())
    .map_err(|err| {
        error!("invite: deactivate failed, err={}", err);
        TexhubError::InviteCheckFailed
    })
}

/// 读机项目名与所有者 id，用于预览。
fn load_project_meta(project_id: &str) -> Result<(String, i64), TexhubError> {
    use crate::model::diesel::tex::custom_tex_models::TexProject;
    use crate::model::diesel::tex::tex_schema::tex_project as proj_table;
    let row = proj_table::table
        .into_boxed::<diesel::pg::Pg>()
        .filter(proj_table::project_id.eq(project_id.to_owned()))
        .limit(1)
        .first::<TexProject>(&mut get_connection())
        .map_err(|_| TexhubError::InviteInvalid)?;
    Ok((row.nickname.clone(), row.user_id))
}

/// 读机用户昵称，失败时返回 None（预览页允许昵称缺失，不该因此拒绝入伙）。
fn load_user_nickname(user_id: i64) -> Option<String> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as editor_table;
    let row = editor_table::table
        .into_boxed::<diesel::pg::Pg>()
        .filter(editor_table::user_id.eq(user_id))
        .limit(1)
        .first::<crate::model::diesel::tex::custom_tex_models::TexProjEditor>(&mut get_connection())
        .ok()?;
    Some(row.nickname)
}

/// 只有项目 Owner 才能签发/撤销邀请。
fn ensure_project_owner(project_id: &str, user_id: i64) -> Result<(), TexhubError> {
    use crate::model::diesel::tex::tex_schema::tex_project as proj_table;
    let owner_id = proj_table::table
        .into_boxed::<diesel::pg::Pg>()
        .filter(proj_table::project_id.eq(project_id.to_owned()))
        .limit(1)
        .select(proj_table::user_id)
        .first::<i64>(&mut get_connection())
        .map_err(|_| TexhubError::ProjAccessUnresolvable)?;
    if owner_id != user_id {
        return Err(TexhubError::InviteNotOwner);
    }
    Ok(())
}

fn to_resp(row: &TexProjInvite, token: String) -> TexInviteResp {
    TexInviteResp {
        project_id: row.project_id.clone(),
        token,
        role_id: row.role_id,
        expire_at: row.expire_at,
        max_uses: row.max_uses,
        used_count: row.used_count,
        created_time: row.created_time,
    }
}
