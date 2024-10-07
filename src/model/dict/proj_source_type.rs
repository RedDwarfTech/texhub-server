#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum ProjSourceType {
    Default = 0,
    TeXHubNew = 1,
    TeXHubTemplate = 2,
    GitHubImport = 3,
    LocalImport = 4,
    Copied = 5,
}

impl From<ProjSourceType> for i32 {
    fn from(collar_status: ProjSourceType) -> Self {
        match collar_status {
            ProjSourceType::Default => 0,
            ProjSourceType::TeXHubNew => 1,
            ProjSourceType::TeXHubTemplate => 2,
            ProjSourceType::GitHubImport => 3,
            ProjSourceType::LocalImport => 4,
            ProjSourceType::Copied => 5,
        }
    }
}