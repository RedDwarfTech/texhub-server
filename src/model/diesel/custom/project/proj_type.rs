#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum ProjType {
    All = 1,
    Shared = 2,
    Archived = 3,
    Trash = 4,
    Deleted = 5,
}

impl From<ProjType> for i32 {
    fn from(file_type: ProjType) -> Self {
        match file_type {
            ProjType::All => 1,
            ProjType::Shared => 2,
            ProjType::Archived => 3,
            ProjType::Trash => 4,
            ProjType::Deleted => 5,
        }
    }
}