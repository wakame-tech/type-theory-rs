use crate::types::Id;
use std::collections::HashMap;

pub struct Issuer {
    value: u8,
    set: HashMap<Id, String>,
    pub nest: usize,
}

impl Default for Issuer {
    fn default() -> Self {
        Issuer {
            value: b'a',
            set: HashMap::new(),
            nest: 0,
        }
    }
}

impl Issuer {
    fn next(&mut self) -> String {
        let id = self.value;
        self.value += 1;
        format!("{}", id as char)
    }

    /// get or create
    pub fn name(&mut self, id: Id) -> String {
        if let Some(name) = self.set.get(&id) {
            name.clone()
        } else {
            let name = self.next();
            self.set.insert(id, name.clone());
            name
        }
    }
}
