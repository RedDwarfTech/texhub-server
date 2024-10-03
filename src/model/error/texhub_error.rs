use rust_wheel::model::error::error_response::ErrorResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TexhubError {
    #[error("超出最大解压尺寸")]
    ExceedMaxUnzipSize,
    #[error("非预期的文件类型")]
    UnexpectFileType,
}

impl ErrorResponse for TexhubError {
    fn error_code(&self) -> &str {
        match self {
            TexhubError::ExceedMaxUnzipSize => "0040010001",
            TexhubError::UnexpectFileType => "0040010002",
        }
    }

    fn error_message(&self) -> &str {
        match self {
            TexhubError::ExceedMaxUnzipSize => "超出最大解压尺寸",
            TexhubError::UnexpectFileType => "非预期的文件类型",
        }
    }

    fn error_code_en(&self) -> &str {
        match self {
            TexhubError::ExceedMaxUnzipSize => "EXCEED_MAX_UNZIP_SIZE",
            TexhubError::UnexpectFileType => "UNEXPECT_FILE_TYPE"
        }
    }
}
