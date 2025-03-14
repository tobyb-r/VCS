use std::fs::File;
use std::{collections::HashMap, io};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use super::ObjectState;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DirHash(#[serde(with = "hex::serde")] pub [u8; 20]);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileHash(#[serde(with = "hex::serde")] pub [u8; 20]);

// type objects in the file tree
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
    pub refcount: i32,
    #[serde(skip)]
    pub state: FileState,
}

impl FileObject {
    pub fn new() -> Self {
        Self {
            refcount: 0,
            state: FileState::New("README.md".to_string()),
        }
    }
    // load object from the repo directory using its hash
    pub fn from_hash(hash: &FileHash) -> Self {
        unimplemented!()
    }
}

// hash file in working tree
pub fn hash_file(mut file: File) -> Result<FileHash> {
    let mut hasher = Sha1::new();

    io::copy(&mut file, &mut hasher)?;

    Ok(FileHash(hasher.finalize()[..].try_into().unwrap()))
}

#[derive(Serialize, Deserialize)]
pub struct DirObject {
    pub objs: HashMap<String, Object>,
    pub refcount: i32,
    #[serde(skip)]
    pub state: ObjectState,
}

impl DirObject {
    pub fn new(filehash: FileHash) -> Self {
        let mut objs = HashMap::new();
        objs.insert("default".to_string(), Object::File(filehash));

        Self {
            objs,
            refcount: 0,
            state: ObjectState::New,
        }
    }

    // load object from the repo directory using its hash
    pub fn from_hash(hash: &DirHash) -> Self {
        unimplemented!()
    }

    // hash object
    pub fn hash(&self) -> DirHash {
        let mut hasher = Sha1::new();

        for (key, value) in &self.objs {
            hasher.update(key);
            hasher.update(match value {
                Object::File(x) => x.0,
                Object::Dir(x) => x.0,
            });
        }

        DirHash(hasher.finalize()[..].try_into().unwrap())
    }
}
