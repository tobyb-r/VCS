// state of an object in the file system
// used when saving our changes to the .mid folder
#[derive(Default)]
pub enum ObjectState {
    // existing object doesnt need to be stored
    #[default] // for serde(skip)
    Existing,
    // new object needs to be stored
    New,
}
