use std::{fmt::Display, sync::atomic::AtomicUsize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Primitive(String),
    TypeVar(String, Option<Box<Type>>),
    // arg ret
    Lambda(Box<Type>, Box<Type>),
}

impl Type {
    pub fn bool() -> Type {
        Type::Primitive("bool".to_string())
    }

    pub fn int() -> Type {
        Type::Primitive("int".to_string())
    }

    pub fn to(&self, ret: Type) -> Type {
        Type::Lambda(Box::new(self.clone()), Box::new(ret))
    }

    pub fn ret_type(&self) -> Option<Type> {
        if let Type::Lambda(_, ret) = self {
            return Some(*ret.clone());
        }
        None
    }

    pub fn arg_type(&self) -> Option<Type> {
        if let Type::Lambda(arg, _) = self {
            return Some(*arg.clone());
        }
        None
    }
}

static TYPE_VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn typ(t: &str) -> Type {
    if t.starts_with("t") {
        return Type::TypeVar(t.to_string(), None);
    }
    Type::Primitive(t.to_string())
}

pub fn type_var() -> (String, Type) {
    let id = TYPE_VAR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let typ = format!("t{}", id);
    (typ.clone(), Type::TypeVar(typ, None))
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Primitive(s) => write!(f, "{}", s),
            Type::TypeVar(s, typ) => {
                if let Some(v) = typ {
                    write!(f, "{}(={})", s, v)
                } else {
                    write!(f, "{}(=?)", s)
                }
            }
            Type::Lambda(arg, ret) => write!(f, "({} -> {})", arg, ret),
        }
    }
}
