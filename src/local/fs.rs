use std::collections::HashMap;

pub struct ObjHash([u8; 16]);

pub enum Object {
    File(FileObj),
    Dir(DirObj),
}

pub struct FileObj {}

pub struct DirObj {
    objs: HashMap<String, ObjHash>,
}
