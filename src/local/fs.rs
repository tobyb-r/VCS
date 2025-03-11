use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ObjectState;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DirHash(#[serde(with = "hex::serde")] pub [u8; 32]);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileHash(#[serde(with = "hex::serde")] pub [u8; 32]);

#[derive(Serialize, Deserialize)]
pub enum Object {
    File(FileHash),
    Dir(DirHash),
}

pub struct FileObject {
    refcount: i32,
    state: ObjectState,
}

pub struct DirObject {
    objs: HashMap<String, Object>,
    refcount: i32,
    state: ObjectState,
}

impl FileObject {
    // load object from the repo directory using its hash
    pub fn from_hash(hash: &FileHash) -> Self {
        unimplemented!()
    }

    // hash object
    pub fn hash(&self) -> FileHash {
        unimplemented!()
    }
}

impl DirObject {
    // load object from the repo directory using its hash
    pub fn from_hash(hash: &DirHash) -> Self {
        unimplemented!()
    }

    // hash object
    pub fn hash(&self) -> DirHash {
        unimplemented!()
    }
}
