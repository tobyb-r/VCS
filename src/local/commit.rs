use std::collections::HashMap;

use super::ObjHash;

pub struct ComHash([u8; 16]);

pub struct Commit {
    msg: String,
    hash: ComHash,
    prev: ComHash,
    objs: ObjHash,
}
