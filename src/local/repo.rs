use core::net;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fs;
use std::fs::{DirBuilder, File};
use std::pin::Pin;

use anyhow::{bail, Result};
use hex::ToHex;
use serde::{Deserialize, Serialize};

use crate::DIR;

use super::{
    Branch, ComHash, Commit, DirHash, DirObject, FileHash, FileObject, FileState, ObjectState,
};

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub stage: Vec<String>,
    pub head: HeadState,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeadState {
    Branch(String),
    Commit(ComHash),
}

impl Repo {
    // load repo from storage
    pub fn load() -> Result<Self> {
        let file = File::open(".mid/repo.json")?;

        let repo: Self = serde_json::from_reader(file)?;

        if let HeadState::Branch(branch) = &repo.head {
            if !repo.branches.contains_key(branch) {
                bail!("Head branch '{}' not in repository.", branch);
            }
        }

        Ok(repo)
    }

    // initialize repository
    pub fn init() -> Result<Self> {
        if fs::exists(DIR)? {
            bail!("Already in repository");
        }

        let mut db = DirBuilder::new();
        db.recursive(true);
        db.create(".mid/objects/commits")?;
        db.create(".mid/objects/files")?;
        db.create(".mid/objects/dirs")?;

        let main = Branch::new(ComHash([0; 20]));
        let mut branches = HashMap::new();
        branches.insert("main".to_string(), main);

        Ok(Self {
            remote: None,
            branches,
            commits: UnsafeCell::new(HashMap::new()),
            files: UnsafeCell::new(HashMap::new()),
            dirs: UnsafeCell::new(HashMap::new()),
            head: HeadState::Branch("main".to_string()),
            stage: Vec::new(),
        })
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_dir(&self, hash: DirHash) -> &DirObject {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.dirs.get() }
            .entry(hash)
            .or_insert_with_key(|hash| Box::pin(DirObject::from_hash(hash)))
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_file(&self, hash: FileHash) -> &FileObject {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.files.get() }
            .entry(hash)
            .or_insert_with_key(|hash| Box::pin(FileObject::from_hash(hash)))
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_commit(&self, hash: ComHash) -> &Commit {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.commits.get() }
            .entry(hash)
            .or_insert_with_key(|hash| Box::pin(Commit::from_hash(hash)))
    }

    // get the current head commit
    pub fn get_head(&self) -> ComHash {
        match &self.head {
            HeadState::Branch(name) => self.branches[name].head,
            HeadState::Commit(hash) => *hash,
        }
    }

    pub fn stage(&mut self, mut paths: Vec<String>) {
        // TODO: validation
        self.stage.append(&mut paths);
    }

    pub fn append_commit(&mut self) {
        let newfile = FileObject::new();
        let filehash = super::hash_file(File::open("README.md").unwrap()).unwrap();
        let newdir = DirObject::new(filehash);
        let new = Commit::new("new".to_string(), self.get_head(), newdir.hash());

        match &self.head {
            HeadState::Branch(branch) => {
                self.branches.get_mut(branch).unwrap().head = new.hash();
            }
            HeadState::Commit(comhash) => {
                self.head = HeadState::Commit(new.hash());
            }
        }

        self.files.get_mut().insert(filehash, Box::pin(newfile));
        self.dirs.get_mut().insert(newdir.hash(), Box::pin(newdir));
        self.commits.get_mut().insert(new.hash(), Box::pin(new));
    }

    // store any changes to the repo
    pub fn save(&self) -> Result<()> {
        let file = File::create(".mid/repo.json")?;
        serde_json::to_writer_pretty(file, self)?;

        // write commits
        // SAFETY: we dont mutate the unsafecell here
        for (key, value) in unsafe { &*self.commits.get() } {
            if let ObjectState::New = value.state {
                let path = format!(".mid/objects/commits/{}.json", key.0.encode_hex::<String>());

                if fs::exists(&path)? {
                    continue;
                }

                let com_file = File::create(&path)?;

                serde_json::to_writer_pretty(com_file, &*value.as_ref())?;
            }
        }

        // write dirs
        // SAFETY: we dont mutate the unsafecell here
        for (key, value) in unsafe { &*self.dirs.get() } {
            if let ObjectState::New = value.state {
                let path = format!(".mid/objects/dirs/{}.json", key.0.encode_hex::<String>());

                if fs::exists(&path)? {
                    continue;
                }

                let com_file = File::create(&path)?;

                serde_json::to_writer_pretty(com_file, &*value.as_ref())?;
            }
        }

        // write files
        // SAFETY: we dont mutate the unsafecell here
        for (key, value) in unsafe { &*self.files.get() } {
            if let FileState::New(inpath) = &value.state {
                let path = format!(".mid/objects/files/{}", key.0.encode_hex::<String>());

                if fs::exists(&path)? {
                    continue;
                }

                let mut db = DirBuilder::new();
                db.recursive(true);

                db.create(&path)?;

                let com_file = File::create(format!("{}/info.json", &path))?;

                serde_json::to_writer_pretty(com_file, &*value.as_ref())?;

                // TODO compression
                fs::copy(inpath, format!("{}/FILE", &path))?;
            }
        }

        Ok(())
    }
}
