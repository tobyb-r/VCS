use std::collections::HashMap;

use super::{DirHash, ObjectState};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ComHash([u8; 32]);

pub struct Commit {
    msg: String,
    hash: ComHash,
    prev: ComHash,
    objs: DirHash,
    state: ObjectState,
}

impl Commit {
    // load object from the repo directory using its hash
    pub fn from_hash(hash: &ComHash) -> Self {
        unimplemented!()
    }

    // hash object
    pub fn hash(&self) -> ComHash {
        unimplemented!()
    }
}
