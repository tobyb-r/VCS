use std::collections::HashMap;

use anyhow::Result;

use crate::FOLDER;

use super::{Branch, ComHash, Commit, DirObj, ObjHash, Object};

pub struct Repo {
    name: String,
    remote: String,
    branches: Vec<Branch>,
    commits: HashMap<ComHash, Commit>,
    objects: HashMap<ObjHash, Object>,
    stage: Option<DirObj>,
    head: HeadState, // todo
}

pub enum HeadState {
    Branch(String),
    Commit(ComHash),
}

impl Repo {
    pub fn load() -> Result<Self> {
        unimplemented!();
    }

    pub fn save(&self) -> Result<()> {
        unimplemented!();
    }
}
