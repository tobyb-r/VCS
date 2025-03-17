use std::ffi::OsString;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
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

// type of objects in the file tree
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
    New(PathBuf),
}

#[derive(Serialize, Deserialize)]
pub struct FileObject {
    pub permissions: u32,
    #[serde(skip)]
    pub state: FileState,
}

impl FileObject {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            permissions: 0,
            state: FileState::New(path.as_ref().to_path_buf()),
        }
    }
}

// find hash of file in working tree
pub fn hash_file(path: impl AsRef<Path>) -> Result<FileHash> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();

    io::copy(&mut file, &mut hasher)?;

    Ok(FileHash(hasher.finalize()[..].try_into().unwrap()))
}

pub fn get_permissions(file: File) {
    todo!()
}

// module for serializing and deserializing the dir objects
// serde doesn't support OsString on its own but String is annoying to work with
mod sd {
    use serde::{ser::SerializeMap, Deserialize, Deserializer, Serializer};
    use std::{collections::HashMap, ffi::OsString};

    use super::Object;

    pub fn serialize<S>(map: &HashMap<OsString, Object>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sermap = ser.serialize_map(Some(map.len()))?;

        for (k, v) in map {
            sermap.serialize_entry(k.to_str().unwrap(), v)?;
        }

        sermap.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<OsString, Object>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the string as a regular string first
        let s: HashMap<String, Object> = Deserialize::deserialize(deserializer)?;

        // Convert the string into an OsString
        Ok(s.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DirObject {
    #[serde(with = "sd")]
    pub objs: HashMap<OsString, Object>,
    #[serde(skip)]
    pub state: ObjectState,
}

impl DirObject {
    pub fn new(filehash: FileHash) -> Self {
        let mut objs = HashMap::new();
        objs.insert("default".to_string().into(), Object::File(filehash));

        Self {
            objs,
            state: ObjectState::New,
        }
    }

    // hash object
    pub fn hash(&self) -> DirHash {
        let mut hasher = Sha1::new();

        for (key, value) in &self.objs {
            hasher.update(key.as_bytes());
            hasher.update(match value {
                Object::File(x) => x.0,
                Object::Dir(x) => x.0,
            });
        }

        DirHash(hasher.finalize()[..].try_into().unwrap())
    }
}
