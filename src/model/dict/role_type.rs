#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum RoleType {
    Owner = 1,
    Collarboartor = 2,
}

impl From<RoleType> for i32 {
    fn from(file_type: RoleType) -> Self {
        match file_type {
            RoleType::Owner => 1,
            RoleType::Collarboartor => 2,
        }
    }
}