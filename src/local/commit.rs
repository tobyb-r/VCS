use hex::ToHex;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use super::{DirHash, ObjectState};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComHash(#[serde(with = "hex::serde")] pub [u8; 16]);

#[derive(Serialize, Deserialize)]
pub struct Commit {
    msg: String,
    prev: ComHash,
    objs: DirHash,
    #[serde(skip)]
    state: ObjectState,
}

impl Commit {
    // load object from the repo directory using its hash
    pub fn from_hash(hash: &ComHash) -> Self {
        unimplemented!()
    }

    // hash object
    pub fn hash(&self) -> ComHash {
        let mut hasher = Sha1::new();
        hasher.update(self.prev.0);
        hasher.update(self.objs.0);
        return ComHash(hasher.finalize()[..].try_into().unwrap());
    }
}
