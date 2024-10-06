use rust_wheel::model::error::error_response::ErrorResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TexhubError {
    #[error("超出最大解压尺寸")]
    ExceedMaxUnzipSize,
    #[error("非预期的文件类型")]
    UnexpectFileType,
    #[error("未找到用户配置")]
    UserConfigMissing,
    #[error("未找到用户GitHub配置")]
    GithubConfigMissing,
    #[error("获取Github仓库大小失败")]
    FetchGithubRepoSizeFailed,
    #[error("超出Github仓库大小限制")]
    ExceedeGithubRepoSize,
}

impl ErrorResponse for TexhubError {
    fn error_code(&self) -> &str {
        match self {
            TexhubError::ExceedMaxUnzipSize => "0040010001",
            TexhubError::UnexpectFileType => "0040010002",
            TexhubError::UserConfigMissing => "0040010003",
            TexhubError::GithubConfigMissing => "0040010004",
            TexhubError::FetchGithubRepoSizeFailed => "0040010005",
            TexhubError::ExceedeGithubRepoSize => "0040010006",
        }
    }

    fn error_message(&self) -> &str {
        match self {
            TexhubError::ExceedMaxUnzipSize => "超出最大解压尺寸",
            TexhubError::UnexpectFileType => "非预期的文件类型",
            TexhubError::UserConfigMissing => "用户配置缺失",
            TexhubError::GithubConfigMissing => "GitHub配置缺失",
            TexhubError::FetchGithubRepoSizeFailed => "获取Github仓库大小失败",
            TexhubError::ExceedeGithubRepoSize => "超出Github仓库大小限制",
        }
    }

    fn error_code_en(&self) -> &str {
        match self {
            TexhubError::ExceedMaxUnzipSize => "EXCEED_MAX_UNZIP_SIZE",
            TexhubError::UnexpectFileType => "UNEXPECT_FILE_TYPE",
            TexhubError::UserConfigMissing => "USER_CONFIG_MISSING",
            TexhubError::GithubConfigMissing => "GITHUB_CONFIG_MISSING",
            TexhubError::FetchGithubRepoSizeFailed => "FETCH_GITHUB_REPO_SIZE_FAILED",
            TexhubError::ExceedeGithubRepoSize => "EXCEED_GITHUB_REPO_SIZE",
        }
    }
}
