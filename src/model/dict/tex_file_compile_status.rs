#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum TeXFileCompileStatus {
    Waiting = 0,
    Compiling = 1,
    Compiled = 2,
    Expired = 3,
}

impl From<TeXFileCompileStatus> for i32 {
    fn from(collar_status: TeXFileCompileStatus) -> Self {
        match collar_status {
            TeXFileCompileStatus::Waiting => 0,
            TeXFileCompileStatus::Compiling => 1,
            TeXFileCompileStatus::Compiled => 2,
            TeXFileCompileStatus::Expired => 3,
        }
    }
}