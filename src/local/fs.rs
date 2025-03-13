use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use super::ObjectState;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DirHash(#[serde(with = "hex::serde")] pub [u8; 16]);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileHash(#[serde(with = "hex::serde")] pub [u8; 16]);

#[derive(Serialize, Deserialize)]
pub enum Object {
    File(FileHash),
    Dir(DirHash),
}

// state of a file in the file system
// used when saving our changes to the .mid folder
#[derive(Default)]
pub enum FileState {
    // existing object doesnt need to be changed
    #[default] // for serde(skip)
    Existing,
    // new object that needs to be stored
    // field contains path that the file needs to be stored from
    New(String),
    // info about object changed and needs to be stored in memory
    // rn the only thing that can change is the refcount
    Updated,
    // object has been marked to be deleted
    Deleted,
}

#[derive(Serialize, Deserialize)]
pub struct FileObject {
    refcount: i32,
    #[serde(skip)]
    state: FileState,
}

#[derive(Serialize, Deserialize)]
pub struct DirObject {
    objs: HashMap<String, Object>,
    refcount: i32,
    #[serde(skip)]
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
