const MAX_FILE_NAME_LENGTH: usize = 255;

#[derive(Debug, Clone)]
pub struct FileNameValidationError {
    pub code: &'static str,
    pub message: &'static str,
}

fn is_allowed_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '_'
        || c == '-'
        || c == '.'
        || ('\u{4e00}'..='\u{9fff}').contains(&c)
}

/// 严格文件名校验：字母/数字/中文/下划线/连字符/点；禁止空格与其它符号。
/// 成功返回 trim 后的名称。
pub fn validate_file_name(raw: &str) -> Result<String, FileNameValidationError> {
    let name = raw.trim().to_string();
    if name.is_empty() {
        return Err(FileNameValidationError {
            code: "INVALID_FILE_NAME",
            message: "file/folder name is empty",
        });
    }
    if name == "." || name == ".." {
        return Err(FileNameValidationError {
            code: "INVALID_FILE_NAME_RESERVED",
            message: "file/folder name cannot be . or ..",
        });
    }
    if name.chars().count() > MAX_FILE_NAME_LENGTH {
        return Err(FileNameValidationError {
            code: "INVALID_FILE_NAME_LENGTH",
            message: "file/folder name exceeds 255 characters",
        });
    }
    if !name.chars().all(is_allowed_char) {
        return Err(FileNameValidationError {
            code: "INVALID_FILE_NAME_CHARS",
            message: "file/folder name contains invalid characters",
        });
    }
    Ok(name)
}
