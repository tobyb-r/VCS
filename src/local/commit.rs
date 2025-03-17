use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use super::{DirHash, ObjectState};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComHash(#[serde(with = "hex::serde")] pub [u8; 20]);

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub msg: String,
    pub prev: ComHash,
    pub objs: DirHash,
    #[serde(skip)]
    pub state: ObjectState,
}

impl Commit {
    pub fn new(msg: String, prev: ComHash, objs: DirHash) -> Self {
        Self {
            msg,
            prev,
            objs,
            state: ObjectState::New,
        }
    }

    // hash object
    pub fn hash(&self) -> ComHash {
        let mut hasher = Sha1::new();
        hasher.update(self.prev.0);
        hasher.update(self.objs.0);
        ComHash(hasher.finalize()[..].try_into().unwrap())
    }
}
