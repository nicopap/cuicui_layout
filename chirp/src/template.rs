use bevy::utils::HashMap;

use crate::parser::StateCheckpoint;

#[derive(Default, Debug)]
pub struct Templates<'a>(HashMap<&'a [u8], StateCheckpoint>);
impl<'a> Templates<'a> {
    pub fn new() -> Self {
        Templates::default()
    }
    pub fn insert(&mut self, name: &'a [u8], state: StateCheckpoint) {
        self.0.insert(name, state);
    }
    pub fn get(&self, name: &'a [u8]) -> Option<StateCheckpoint> {
        self.0.get(name).copied()
    }
}
