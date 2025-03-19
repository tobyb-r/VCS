use anyhow::{bail, Context, Result};
use hex::ToHex;
use serde::{Deserialize, Serialize};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::{DirBuilder, File};
use std::path::{Components, Path, PathBuf};
use std::pin::Pin;

use super::{
    hash_file, Branch, ComHash, Commit, DirHash, DirObject, FileHash, FileObject, FileState,
    Object, ObjectState,
};

#[derive(Serialize, Deserialize)]
pub struct Repo {
    pub remote: Option<String>,            // url of remote
    pub branches: HashMap<String, Branch>, // branch names to branch
    pub head: HeadState,                   // detached commit or a branch name

    // lazily loaded store of objects
    #[serde(skip)]
    commits: UnsafeCell<HashMap<ComHash, Pin<Box<Commit>>>>,
    #[serde(skip)]
    files: UnsafeCell<HashMap<FileHash, Pin<Box<FileObject>>>>,
    #[serde(skip)]
    dirs: UnsafeCell<HashMap<DirHash, Pin<Box<DirObject>>>>,

    // staging area
    index: Option<DirHash>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadState {
    Branch(String),
    Commit(ComHash),
}

impl Repo {
    // load repo from storage
    pub fn load() -> Result<Self> {
        let file = File::open(".mid/repo.json")?;

        let repo: Self = serde_json::from_reader(file)?;

        // validation
        if let HeadState::Branch(branch) = &repo.head {
            if !repo.branches.contains_key(branch) {
                bail!("Head branch '{}' not in repository.", branch);
            }
        }

        Ok(repo)
    }

    // initialize repository
    pub fn init() -> Result<Self> {
        if fs::exists(".mid")? {
            bail!("Already in repository");
        }

        // creating dirs for storing objects
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
            // TODO: handle this error properly
            .or_insert_with(|| {
                Box::pin(
                    self.dir_from_hash(hash)
                        .with_context(|| {
                            format!("Failed loading dir {}", hash.0.encode_hex::<String>())
                        })
                        .unwrap(),
                )
            })
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_file(&self, hash: FileHash) -> &FileObject {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.files.get() }
            .entry(hash)
            .or_insert_with(|| Box::pin(self.file_from_hash(hash)))
    }

    // get object from hashmap, load it from storage if it isn't there
    pub fn get_commit(&self, hash: ComHash) -> &Commit {
        // SAFETY: access is unique because we never leak references to the hashmap
        // SAFETY: references will stay valid because of pin
        unsafe { &mut *self.commits.get() }
            .entry(hash)
            // TODO: handle this error properly
            .or_insert_with(|| {
                Box::pin(
                    self.commit_from_hash(hash)
                        .with_context(|| {
                            format!("Failed loading commit {}", hash.0.encode_hex::<String>())
                        })
                        .unwrap(),
                )
            })
    }

    // get the current head commit
    pub fn get_head(&self) -> ComHash {
        match &self.head {
            HeadState::Branch(name) => self.branches[name].head,
            HeadState::Commit(hash) => *hash,
        }
    }

    // create new dir and store it in the repo
    // stored in the .mid dir at the end
    fn new_dir(&mut self, objs: HashMap<OsString, Object>) -> DirHash {
        let dir = DirObject {
            objs,
            state: ObjectState::New,
        };
        let hash = dir.hash();
        self.dirs.get_mut().entry(hash).or_insert(Box::pin(dir));
        hash
    }

    // create new commit and store it in the repo
    // stored in the .mid dir at the end
    fn new_commit(&mut self, msg: String, prev: ComHash, objs: DirHash) -> ComHash {
        let commit = Commit {
            msg,
            prev,
            objs,
            state: ObjectState::New,
        };
        let hash = commit.hash();
        self.commits
            .get_mut()
            .entry(hash)
            .or_insert(Box::pin(commit));
        hash
    }

    // load object from the repo directory using its hash
    pub fn commit_from_hash(&self, hash: ComHash) -> Result<Commit> {
        // [0;20] represents hash of commit 0
        if hash.0 == [0; 20] {
            Ok(Commit {
                msg: "init".to_string(),
                prev: ComHash([0; 20]),
                objs: DirHash([0; 20]),
                state: ObjectState::Existing,
            })
        } else {
            let path = format!(
                ".mid/objects/commits/{}.json",
                hash.0.encode_hex::<String>()
            );

            if !fs::exists(&path)? {
                bail!("File doesn't exist");
            }

            let file = File::open(path)?;

            Ok(serde_json::from_reader(file)?)
        }
    }

    // load object from the repo directory using its hash
    pub fn dir_from_hash(&self, hash: DirHash) -> Result<DirObject> {
        if hash.0 == [0; 20] {
            Ok(DirObject {
                objs: HashMap::new(),
                state: ObjectState::Existing,
            })
        } else {
            let path = format!(".mid/objects/dirs/{}.json", hash.0.encode_hex::<String>());

            if !fs::exists(&path)? {
                bail!("File doesn't exist");
            }

            let file = File::open(path)?;

            Ok(serde_json::from_reader(file)?)
        }
    }

    // load object from the repo directory using its hash
    pub fn file_from_hash(&self, hash: FileHash) -> FileObject {
        unimplemented!()
    }

    // stage entire folder or file
    pub fn index_path(&mut self, path: impl AsRef<Path>) -> Result<Option<Object>> {
        if !path.as_ref().try_exists()? {
            Ok(None) // ts doesn't exist
        } else if path.as_ref().is_dir() {
            let mut objs = HashMap::new();
            for entry in fs::read_dir(path.as_ref())? {
                let entry = entry?;
                if let Some(object) = self.index_path(entry.path())? {
                    objs.insert(entry.file_name(), object);
                }
            }

            Ok(Some(Object::Dir(self.new_dir(objs))))
        } else {
            // path is file
            let hash = hash_file(path.as_ref())?;
            let file = FileObject::new(path.as_ref());
            self.files.get_mut().insert(hash, Box::pin(file));
            Ok(Some(Object::File(hash)))
        }
    }

    // add list of paths to index
    // TODO: support wildcards
    pub fn index_paths(&mut self, paths: Vec<impl AsRef<Path>>) -> Result<()> {
        let canon_dot = fs::canonicalize(".")?;

        // canonicalize all paths relative to current directory
        // fixes any bs
        let paths = paths
            .iter()
            .map::<Result<_>, _>(|path| {
                Ok(fs::canonicalize(path)?
                    .strip_prefix(&canon_dot)?
                    .to_path_buf())
            })
            .collect::<Result<Vec<_>>>()?;

        // convert paths into vec of (pathbuf, components iterator)
        let paths = paths
            .iter()
            .map(|pb| (pb, pb.components()))
            .collect::<Vec<(_, _)>>();

        self.index = Some(
            step(
                self,
                Some(self.index.unwrap_or(self.get_commit(self.get_head()).objs)),
                paths,
            )?
            .unwrap()
            .get_dir()
            .unwrap(),
        );

        // recursively iterates through paths staged to be commited
        pub fn step(
            repo: &mut Repo,
            // hash of the object in HEAD representing this dir
            // None if we are in a new dir
            dirhash: Option<DirHash>,
            // all the paths in one subpath and the state of their component iterators
            paths: Vec<(&PathBuf, Components<'_>)>,
        ) -> Result<Option<Object>> {
            // maps paths off this directory to the paths param for the recursive call
            let mut subdirs: HashMap<&OsStr, Vec<(&PathBuf, Components<'_>)>> = HashMap::new();

            for mut path in paths.into_iter() {
                if let Some(comp) = path.1.next() {
                    subdirs
                        .entry(comp.as_os_str())
                        .or_default()
                        .push((path.0, path.1));
                } else {
                    // we were at the end of the path for this
                    return repo.index_path(path.0);
                };
            }

            // if their is an existing dirobject then we keep the objects it contained
            let mut objs = if let Some(hash) = dirhash {
                repo.get_dir(hash).objs.clone()
            } else {
                HashMap::new()
            };

            // recursive call on all subpaths
            // insert new objects into the objs
            for (k, v) in subdirs {
                if let Some(new) = step(repo, objs.get(k).and_then(Object::get_dir), v)? {
                    objs.insert(k.to_os_string(), new);
                } else {
                    // no file at that path
                    objs.remove(k); // remove entry if it was in dir but was deleted
                }
            }

            Ok(Some(Object::Dir(repo.new_dir(objs))))
        }

        Ok(())
    }

    // takes dirobj out of index and commits it
    pub fn commit_index(&mut self, msg: String) -> Result<()> {
        let Some(index) = self.index.take() else {
            bail!("Index empty")
        };

        self.append_commit(msg, index);

        Ok(())
    }

    // appends commit to head, moves head to point at new commit
    pub fn append_commit(&mut self, msg: String, new_dir: DirHash) {
        let new_head = self.new_commit(msg, self.get_head(), new_dir);

        match &self.head {
            HeadState::Commit(comhash) => {
                self.head = HeadState::Commit(new_head);
            }
            HeadState::Branch(branch) => {
                *self.branches.get_mut(branch).unwrap() = Branch { head: new_head }
            }
        }
    }

    // store any changes to the repo
    pub fn save(&self) -> Result<()> {
        let file = File::create(".mid/repo.json")?;
        serde_json::to_writer_pretty(file, self)?;

        // write commits to .mid dir
        // SAFETY: we dont mutate the unsafecell while it is borrowed
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

        // write dirs to .mid dir
        // SAFETY: we dont mutate the unsafecell while it is borrowed
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

        // write files to .mid dir
        // SAFETY: we dont mutate the unsafecell while it is borrowed
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

                // TODO: compression
                fs::copy(inpath, format!("{}/FILE", &path))?;
            }
        }

        Ok(())
    }
}
