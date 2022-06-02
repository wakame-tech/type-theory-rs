use std::collections::HashMap;

pub type Id = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Variable {
        id: Id,
        instance: Option<Id>,
    },
    Operator {
        id: Id,
        name: String,
        types: Vec<Id>,
    },
}

pub struct Issuer {
    pub value: u8,
    pub set: HashMap<Id, String>,
}

impl Issuer {
    pub fn new(start: char) -> Self {
        Issuer {
            value: start as u8,
            set: HashMap::new(),
        }
    }

    fn next(&mut self) -> String {
        let id = self.value;
        self.value += 1;
        format!("{}", id as char)
    }

    /// get or create
    fn name(&mut self, id: Id) -> String {
        if let Some(name) = self.set.get(&id) {
            name.clone()
        } else {
            let name = self.next();
            self.set.insert(id, name.clone());
            name
        }
    }
}

impl Type {
    pub fn var(id: Id) -> Type {
        Type::Variable { id, instance: None }
    }

    pub fn fun(id: Id, arg: Id, ret: Id) -> Type {
        Type::Operator {
            id,
            name: "->".to_string(),
            types: vec![arg, ret],
        }
    }

    pub fn op(id: Id, name: &str, types: &[Id]) -> Type {
        Type::Operator {
            id,
            name: name.to_string(),
            types: types.to_vec(),
        }
    }

    pub fn id(&self) -> Id {
        match self {
            Type::Variable { id, .. } => *id,
            Type::Operator { id, .. } => *id,
        }
    }

    pub fn set_instance(&mut self, id: Id) {
        match self {
            Type::Variable { instance, .. } => {
                *instance = Some(id);
            }
            _ => panic!("set_instance called on non-variable type"),
        }
    }

    pub fn as_string(&self, a: &Vec<Type>, issuer: &mut Issuer) -> String {
        match self {
            &Type::Variable {
                instance: Some(inst),
                ..
            } => a[inst].as_string(a, issuer),
            &Type::Variable { .. } => issuer.name(self.id()),
            &Type::Operator {
                ref types,
                ref name,
                ..
            } => match types.len() {
                0 => name.clone(),
                2 => {
                    let l = a[types[0]].as_string(a, issuer);
                    let r = a[types[1]].as_string(a, issuer);
                    format!("({} {} {})", l, name, r)
                }
                _ => {
                    let mut coll = vec![];
                    for v in types {
                        coll.push(a[*v].as_string(a, issuer));
                    }
                    format!("{} {}", name, coll.join(" "))
                }
            },
        }
    }
}

pub fn new_variable(a: &mut Vec<Type>) -> Id {
    let id = a.len();
    a.push(Type::var(id));
    id
}

pub fn new_function(a: &mut Vec<Type>, arg: Id, ret: Id) -> Id {
    let id = a.len();
    let typ = Type::op(id, "->", &[arg, ret]);
    a.push(typ);
    id
}

pub fn new_operator(a: &mut Vec<Type>, name: &str, types: &[Id]) -> Id {
    let id = a.len();
    let typ = Type::op(id, name, types);
    a.push(typ);
    id
}
