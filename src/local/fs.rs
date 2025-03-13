use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

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
        let mut hasher = Sha1::new();

        for (key, value) in self.objs.iter() {
            hasher.update(key);
            hasher.update(match value {
                Object::File(x) => x.0,
                Object::Dir(x) => x.0,
            });
        }

        return DirHash(hasher.finalize()[..].try_into().unwrap());
    }
}
