use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::pin::Pin;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::DIR;

use super::{Branch, ComHash, Commit, DirHash, DirObject, FileHash, FileObject};

#[derive(Serialize, Deserialize)]
pub struct Repo {
    name: String,
    remote: String,
    branches: HashMap<String, Branch>,

    // lazily loaded store of objects
    #[serde(skip)]
    commits: UnsafeCell<HashMap<ComHash, Pin<Box<Commit>>>>,
    #[serde(skip)]
    files: UnsafeCell<HashMap<FileHash, Pin<Box<FileObject>>>>,
    #[serde(skip)]
    dirs: UnsafeCell<HashMap<DirHash, Pin<Box<DirObject>>>>,

    // list of paths
    stage: Option<Vec<String>>,
    head: HeadState,
}

#[derive(Serialize, Deserialize)]
pub enum HeadState {
    Branch(String),
    Commit(ComHash),
}

impl Repo {
    pub fn load() -> Result<Self> {
        unimplemented!();
    }

    pub fn get_dir(&self, hash: &DirHash) -> &DirObject {
        unsafe { &mut *self.dirs.get() }
            .entry(*hash)
            .or_insert_with_key(|hash| Box::pin(DirObject::from_hash(hash)))
    }

    pub fn get_file(&self, hash: &FileHash) -> &FileObject {
        unsafe { &mut *self.files.get() }
            .entry(*hash)
            .or_insert_with_key(|hash| Box::pin(FileObject::from_hash(hash)))
    }

    pub fn get_commit(&self, hash: &ComHash) -> &Commit {
        unsafe { &mut *self.commits.get() }
            .entry(*hash)
            .or_insert_with_key(|hash| Box::pin(Commit::from_hash(hash)))
    }

    pub fn save(&self) -> Result<()> {
        unimplemented!();
    }
}
