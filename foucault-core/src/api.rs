use serde::{Deserialize, Serialize};

pub mod note {
    use super::*;

    use crate::link_repr::Link;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateParam {
        pub name: String,
        pub content: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RenameParam {
        pub id: i64,
        pub name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateContentParam {
        pub id: i64,
        pub content: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateLinksParam {
        pub id: i64,
        pub links: Vec<Link>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ValidateNewTagParam {
        pub id: i64,
        pub tag_id: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddTagParam {
        pub id: i64,
        pub tag_id: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RemoveTagParam {
        pub id: i64,
        pub tag_id: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchWithTagParam {
        pub tag_id: i64,
        pub pattern: String,
    }
}

pub mod tag {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RenameParam {
        pub id: i64,
        pub name: String,
    }
}
