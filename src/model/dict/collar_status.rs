#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum CollarStatus {
    Normal = 1,
    Exit = 2,
}

impl From<CollarStatus> for i32 {
    fn from(collar_status: CollarStatus) -> Self {
        match collar_status {
            CollarStatus::Normal => 1,
            CollarStatus::Exit => 2,
        }
    }
}