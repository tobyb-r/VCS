use core::net;
use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::pin::Pin;
use std::{cell::UnsafeCell, fs};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::DIR;

use super::{Branch, ComHash, Commit, DirHash, DirObject, FileHash, FileObject};

#[derive(Serialize, Deserialize)]
pub struct Repo {
    pub remote: Option<String>,
    pub branches: HashMap<String, Branch>,

    // lazily loaded store of objects
    #[serde(skip)]
    commits: UnsafeCell<HashMap<ComHash, Pin<Box<Commit>>>>,
    #[serde(skip)]
    files: UnsafeCell<HashMap<FileHash, Pin<Box<FileObject>>>>,
    #[serde(skip)]
    dirs: UnsafeCell<HashMap<DirHash, Pin<Box<DirObject>>>>,

    // list of paths
    pub stage: Option<Vec<String>>,
    pub head: HeadState,
}

#[derive(Serialize, Deserialize)]
pub enum HeadState {
    Branch(String),
    Commit(ComHash),
}

impl Repo {
    pub fn load() -> Result<Self> {
        let file = File::open(".mid/repo.json")?;
        Ok(serde_json::from_reader(file)?)
    }

    pub fn get_dir(&self, hash: &DirHash) -> &DirObject {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.dirs.get() }
            .entry(*hash)
            .or_insert_with_key(|hash| Box::pin(DirObject::from_hash(hash)))
    }

    pub fn get_file(&self, hash: &FileHash) -> &FileObject {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.files.get() }
            .entry(*hash)
            .or_insert_with_key(|hash| Box::pin(FileObject::from_hash(hash)))
    }

    pub fn get_commit(&self, hash: &ComHash) -> &Commit {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.commits.get() }
            .entry(*hash)
            .or_insert_with_key(|hash| Box::pin(Commit::from_hash(hash)))
    }

    pub fn init() -> Result<Self> {
        if fs::exists(DIR)? {
            bail!("Already in directory");
        }

        let mut db = DirBuilder::new();
        db.recursive(true);
        // db.create(".mid")?;
        // db.create(".mid/objects")?;
        db.create(".mid/objects/commits")?;
        db.create(".mid/objects/files")?;
        db.create(".mid/objects/dirs")?;

        let main = Branch::new(ComHash([0; 16]));
        let mut branches = HashMap::new();
        branches.insert("main".to_string(), main);

        Ok(Self {
            remote: None,
            branches,
            commits: UnsafeCell::new(HashMap::new()),
            files: UnsafeCell::new(HashMap::new()),
            dirs: UnsafeCell::new(HashMap::new()),
            head: HeadState::Branch("main".to_string()),
            stage: None,
        })
    }

    pub fn save(&self) -> Result<()> {
        let file = File::create(".mid/repo.json")?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}
