use rust_wheel::model::user::login_user_info::LoginUserInfo;

use crate::model::{
    diesel::tex::custom_tex_models::{TexProjFolder, TexProject},
    request::project::query::{
        proj_list_query_params::ProjListQueryParams, proj_query_params::ProjQueryParams,
    },
    response::project::tex_proj_resp::TexProjResp,
};

pub trait ProjSpec {
    fn get_proj_count_by_uid(&self, uid: &i64) -> i64;

    fn get_proj_by_type(
        &self,
        query_params: &ProjQueryParams,
        login_user_info: &LoginUserInfo,
        default_folder: Option<&TexProjFolder>,
    ) -> Vec<TexProjResp>;

    fn get_proj_list(
        &self,
        query_params: &ProjListQueryParams,
        login_user_info: &LoginUserInfo,
    ) -> Vec<TexProject>;
}
