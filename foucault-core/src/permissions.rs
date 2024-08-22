use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Permissions {
    ReadWrite,
    ReadOnly,
}

impl Permissions {
    #[must_use]
    pub fn writable(&self) -> bool {
        match self {
            Permissions::ReadWrite => true,
            Permissions::ReadOnly => false,
        }
    }
}
