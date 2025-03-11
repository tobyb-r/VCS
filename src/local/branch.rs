use serde::{Deserialize, Serialize};

use super::ComHash;

#[derive(Serialize, Deserialize)]
pub struct Branch {
    pub head: ComHash,
}

impl Branch {
    pub fn new(head: ComHash) -> Self {
        Self { head: head }
    }
}
