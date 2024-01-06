use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub name: String,
    pub steam_app_id: u32,
    pub steam_user_id: u64,
    pub steam_username: String,
    pub tags: Vec<String>,
    /// Should always be a valid u128, but some parsers do not support that
    pub id: String,
    pub config: super::Config,
}

impl Metadata {
    pub fn set_id(&mut self, id: u128) {
        self.id = id.to_string()
    }

    pub fn get_id(&self) -> u128 {
        self.id.parse().expect("metadata id must be u128")
    }
}
