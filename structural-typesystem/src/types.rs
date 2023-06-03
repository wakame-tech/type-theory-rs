use std::fmt::Display;

pub type Id = usize;

#[derive(Debug, Clone, Hash)]
pub enum Type {
    Variable {
        id: Id,
        instance: Option<Id>,
    },
    /// ex. function type "->", tuple type ","
    Operator {
        id: Id,
        name: String,
        types: Vec<Id>,
    },
    Record {
        id: Id,
        types: Vec<(String, Type)>,
    },
}

impl Type {
    /// create a new type variable
    pub fn var(id: Id) -> Type {
        Type::Variable { id, instance: None }
    }

    pub fn id(&self) -> Id {
        match self {
            Type::Variable { id, .. } => *id,
            Type::Operator { id, .. } => *id,
            Type::Record { id, .. } => *id,
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
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Operator { id, name, types } => {
                write!(
                    f,
                    "{} #{}({})",
                    name,
                    id,
                    types
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            Type::Variable { id, instance } => {
                if let Some(inst) = instance {
                    write!(f, "#{}", inst)
                } else {
                    write!(f, "#{}", id)
                }
            }
            Type::Record { id, types } => write!(
                f,
                "#{{{}}}",
                types
                    .iter()
                    .map(|t| t.1.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}
