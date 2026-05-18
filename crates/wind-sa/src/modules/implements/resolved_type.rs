use crate::modules::types::WindResolvedType;

impl WindResolvedType {
    pub fn from_builtin_name(name: &str) -> Option<Self> {
        match name {
            "int" => Some(WindResolvedType::Int),
            "float" => Some(WindResolvedType::Float),
            "string" => Some(WindResolvedType::String),
            "char" => Some(WindResolvedType::Char),
            "bool" => Some(WindResolvedType::Bool),
            "None" => Some(WindResolvedType::None),
            "byte" => Some(WindResolvedType::Byte),
            "tag" => Some(WindResolvedType::Tag),
            _ => None,
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            WindResolvedType::Int => "int".to_string(),
            WindResolvedType::Float => "float".to_string(),
            WindResolvedType::String => "string".to_string(),
            WindResolvedType::Char => "char".to_string(),
            WindResolvedType::Bool => "bool".to_string(),
            WindResolvedType::None => "None".to_string(),
            WindResolvedType::Byte => "byte".to_string(),
            WindResolvedType::Tuple(elems) => {
                let names: Vec<String> = elems.iter().map(|e| e.display_name()).collect();
                format!("({})", names.join(", "))
            }
            WindResolvedType::Vec(elem) => format!("vec<{}>", elem.display_name()),
            WindResolvedType::Map(k, v) => format!("map<{}, {}>", k.display_name(), v.display_name()),
            WindResolvedType::Set(elem) => format!("set<{}>", elem.display_name()),
            WindResolvedType::Struct(name) => name.clone(),
            WindResolvedType::Enum(name) => name.clone(),
            WindResolvedType::Tag => "tag".to_string(),
            WindResolvedType::Function { params, ret } => {
                let p: Vec<String> = params.iter().map(|t| t.display_name()).collect();
                format!("fn({}) -> {}", p.join(", "), ret.display_name())
            }
            WindResolvedType::SelfType(s) => s.clone(),
            WindResolvedType::Unknown => "<unknown>".to_string(),
            WindResolvedType::Error => "<error>".to_string(),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(
            self,
            WindResolvedType::Int
                | WindResolvedType::Float
                | WindResolvedType::String
                | WindResolvedType::Char
                | WindResolvedType::Bool
                | WindResolvedType::None
                | WindResolvedType::Byte
                | WindResolvedType::Tag
        )
    }

    pub fn is_container(&self) -> bool {
        matches!(
            self,
            WindResolvedType::Vec(_) | WindResolvedType::Map(_, _) | WindResolvedType::Set(_)
        )
    }

    pub fn is_value_type(&self) -> bool {
        matches!(
            self,
            WindResolvedType::Int
                | WindResolvedType::Float
                | WindResolvedType::Char
                | WindResolvedType::Bool
                | WindResolvedType::None
                | WindResolvedType::Byte
        )
    }
}
