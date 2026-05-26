// types.rs — BQL scalar types. Single source of truth.
use serde::{Deserialize, Serialize};

/// A BQL scalar value. All comparisons are done against these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BqlValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<BqlValue>),
}

impl BqlValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(b) => *b,
            Self::Int(n) => *n != 0,
            Self::Float(f) => *f != 0.0,
            Self::Str(s) => !s.is_empty(),
            Self::List(v) => !v.is_empty(),
        }
    }

    pub fn as_str(&self) -> Option<&str> { if let Self::Str(s) = self { Some(s) } else { None } }
    pub fn as_int(&self) -> Option<i64>  { if let Self::Int(n) = self { Some(*n) } else { None } }
    pub fn as_float(&self) -> Option<f64>{ if let Self::Float(f) = self { Some(*f) } else { None } }

    /// Compare two values for ordering (used in ORDER BY).
    pub fn cmp_for_order(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Int(a),   Self::Int(b))   => a.cmp(b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Self::Str(a),   Self::Str(b))   => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }
}

impl From<&serde_json::Value> for BqlValue {
    fn from(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null        => Self::Null,
            serde_json::Value::Bool(b)     => Self::Bool(*b),
            serde_json::Value::Number(n)   => n.as_i64().map(Self::Int).or_else(|| n.as_f64().map(Self::Float)).unwrap_or(Self::Null),
            serde_json::Value::String(s)   => Self::Str(s.clone()),
            serde_json::Value::Array(arr)  => Self::List(arr.iter().map(Self::from).collect()),
            serde_json::Value::Object(_)   => Self::Null,
        }
    }
}

impl std::fmt::Display for BqlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null    => write!(f, "null"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(n)  => write!(f, "{n}"),
            Self::Float(x)=> write!(f, "{x}"),
            Self::Str(s)  => write!(f, "{s}"),
            Self::List(v) => write!(f, "[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn truthy() { assert!(!BqlValue::Null.is_truthy()); assert!(BqlValue::Str("x".into()).is_truthy()); assert!(!BqlValue::Int(0).is_truthy()); }
    #[test] fn from_json() { assert_eq!(BqlValue::from(&serde_json::json!("hello")), BqlValue::Str("hello".into())); }
}