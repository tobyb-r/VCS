// state of an object in the file system
// used when saving our changes to the .mid folder
#[derive(Default)]
pub enum ObjectState {
    // existing object doesnt need to be changed
    #[default] // for serde(skip)
    Existing,
    // new object needs to be stored
    New,
    // info about object changed and needs to be stored in memory
    // rn the only thing that can change is the refcount
    Updated,
    // object has been marked to be deleted
    Deleted,
}
