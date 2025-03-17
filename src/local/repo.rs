use anyhow::{bail, Result};
use hex::ToHex;
use serde::{Deserialize, Serialize};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::fs::{DirBuilder, File};
use std::path::Path;
use std::pin::Pin;

use crate::DIR;

use super::{
    hash_file, Branch, ComHash, Commit, DirHash, DirObject, FileHash, FileObject, FileState,
    Object, ObjectState,
};

#[derive(Serialize, Deserialize)]
pub struct Repo {
    pub remote: Option<String>, // path of remote
    pub branches: HashMap<String, Branch>,
    pub head: HeadState,

    // lazily loaded store of objects
    #[serde(skip)]
    commits: UnsafeCell<HashMap<ComHash, Pin<Box<Commit>>>>,
    #[serde(skip)]
    files: UnsafeCell<HashMap<FileHash, Pin<Box<FileObject>>>>,
    #[serde(skip)]
    dirs: UnsafeCell<HashMap<DirHash, Pin<Box<DirObject>>>>,

    // staging area
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    index: Option<DirHash>,
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
            index: None,
        })
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_dir(&self, hash: DirHash) -> &DirObject {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.dirs.get() }
            .entry(hash)
            .or_insert_with(|| Box::pin(DirObject::from_hash(hash)))
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_file(&self, hash: FileHash) -> &FileObject {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.files.get() }
            .entry(hash)
            .or_insert_with(|| Box::pin(FileObject::from_hash(hash)))
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_commit(&self, hash: ComHash) -> &Commit {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.commits.get() }
            .entry(hash)
            .or_insert_with(|| Box::pin(Commit::from_hash(hash)))
    }

    // get the current head commit
    pub fn get_head(&self) -> ComHash {
        match &self.head {
            HeadState::Branch(name) => self.branches[name].head,
            HeadState::Commit(hash) => *hash,
        }
    }

    fn new_dir(&mut self, objs: HashMap<OsString, Object>) -> DirHash {
        let dir = DirObject {
            objs,
            state: ObjectState::New,
        };
        let hash = dir.hash();
        self.dirs.get_mut().insert(hash, Box::pin(dir));
        hash
    }

    fn new_commit(&mut self, msg: String, prev: ComHash, objs: DirHash) -> ComHash {
        let commit = Commit {
            msg,
            prev,
            objs,
            state: ObjectState::New,
        };
        let hash = commit.hash();
        self.commits.get_mut().insert(hash, Box::pin(commit));
        hash
    }

    // stage entire folder
    // TODO: only stage necessary
    pub fn index_dir(&mut self, path: impl AsRef<Path>) -> Result<DirHash> {
        let head_rev = self.get_dir(self.get_commit(self.get_head()).objs);
        let mut objs = HashMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;

            if entry.metadata()?.is_file() {
                let hash = hash_file(entry.path())?;
                self.files.get_mut().insert(
                    hash,
                    Box::pin(FileObject::new(entry.path().into_os_string())),
                );

                objs.insert(entry.file_name(), Object::File(hash));
            } else {
                // is dir
                let hash = self.index_dir(entry.path().as_os_str())?;
                objs.insert(entry.file_name(), Object::Dir(hash));
            }
        }

        Ok(self.new_dir(objs))
    }

    // add list of paths to index
    pub fn stage(&mut self, paths: Vec<String>) -> Result<()> {
        self.index = Some(self.index_dir("src")?);

        return Ok(());

        // recursively iterate through directory entries
        // don't add files that arent in the paths or haven't changed
        // maybe use trie

        // TODO: finish this
        todo!();
    }

    pub fn commit_staged(&mut self, msg: String) -> Result<()> {
        let Some(index) = self.index.take() else {
            bail!("Index empty")
        };

        let new_head = self.new_commit(msg, self.get_head(), index);

        self.append_commit(new_head);

        Ok(())
    }

    pub fn append_commit(&mut self, new_head: ComHash) {
        match &self.head {
            HeadState::Commit(comhash) => {
                self.head = HeadState::Commit(new_head);
            }
            HeadState::Branch(branch) => {
                *self.branches.get_mut(branch).unwrap() = Branch { head: new_head }
            }
        }
    }

    // pub fn append_commit(&mut self) {
    //     let newfile = FileObject::new();
    //     let filehash = super::hash_file(File::open("README.md").unwrap()).unwrap();
    //     let newdir = DirObject::new(filehash);
    //     let new = Commit::new("new".to_string(), self.get_head(), newdir.hash());

    //     match &self.head {
    //         HeadState::Branch(branch) => {
    //             self.branches.get_mut(branch).unwrap().head = new.hash();
    //         }
    //         HeadState::Commit(comhash) => {
    //             self.head = HeadState::Commit(new.hash());
    //         }
    //     }

    //     self.files.get_mut().insert(filehash, Box::pin(newfile));
    //     self.dirs.get_mut().insert(newdir.hash(), Box::pin(newdir));
    //     self.commits.get_mut().insert(new.hash(), Box::pin(new));
    // }

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

                serde_json::to_writer_pretty(com_file, &**value)?;
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

                serde_json::to_writer_pretty(com_file, &**value)?;
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

                serde_json::to_writer_pretty(com_file, &**value)?;

                // TODO compression
                fs::copy(inpath, format!("{}/FILE", &path))?;
            }
        }

        Ok(())
    }
}
