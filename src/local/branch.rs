use serde::{Deserialize, Serialize};

use super::ComHash;

#[derive(Serialize, Deserialize)]
pub struct Branch {
    head: ComHash,
}
