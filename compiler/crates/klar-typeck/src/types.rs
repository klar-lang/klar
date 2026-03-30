/// Klar's type representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitives
    Int,
    Float,
    Bool,
    String,
    Byte,
    Unit,

    // Composite
    Struct(String, Vec<(String, Type)>),   // name, fields
    Enum(String, Vec<(String, Vec<Type>)>), // name, variants with field types
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),

    // Function
    Fn(Vec<Type>, Box<Type>),  // params, return

    // Option / Result
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),

    // Type variable (for inference)
    Var(usize),

    // Named type (unresolved)
    Named(String),

    // Error placeholder
    Error,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Var(_))
    }

    pub fn display_name(&self) -> String {
        match self {
            Type::Int => "Int".into(),
            Type::Float => "Float".into(),
            Type::Bool => "Bool".into(),
            Type::String => "String".into(),
            Type::Byte => "Byte".into(),
            Type::Unit => "Unit".into(),
            Type::Struct(name, _) => name.clone(),
            Type::Enum(name, _) => name.clone(),
            Type::List(t) => format!("List[{}]", t.display_name()),
            Type::Map(k, v) => format!("Map[{}, {}]", k.display_name(), v.display_name()),
            Type::Set(t) => format!("Set[{}]", t.display_name()),
            Type::Fn(params, ret) => {
                let ps: Vec<_> = params.iter().map(|p| p.display_name()).collect();
                format!("fn({}) -> {}", ps.join(", "), ret.display_name())
            }
            Type::Option(t) => format!("{}?", t.display_name()),
            Type::Result(t, e) => format!("{} ! {}", t.display_name(), e.display_name()),
            Type::Var(id) => format!("?T{}", id),
            Type::Named(n) => n.clone(),
            Type::Error => "<error>".into(),
        }
    }
}
