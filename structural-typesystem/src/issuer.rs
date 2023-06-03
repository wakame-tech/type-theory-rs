use std::collections::HashMap;

use crate::types::Id;

pub struct Issuer {
    pub value: u8,
    pub set: HashMap<Id, String>,
}

impl Default for Issuer {
    fn default() -> Self {
        Issuer {
            value: 'a' as u8,
            set: HashMap::new(),
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
