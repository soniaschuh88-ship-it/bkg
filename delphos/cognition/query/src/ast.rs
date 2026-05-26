// ast.rs — BQL Abstract Syntax Tree. Single source of truth for query structure.
use serde::{Deserialize, Serialize};
use crate::types::BqlValue;

/// Entity type to query (e.g. "tasks", "agents", "sessions").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityType(pub String);
impl EntityType { pub fn new(s: impl Into<String>) -> Self { Self(s.into()) } }

/// ORDER BY direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderDir { Asc, Desc }

/// A field path within an entity (e.g. "status", "dependency.depth").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldPath(pub Vec<String>);
impl FieldPath {
    pub fn new(path: &str) -> Self { Self(path.split('.').map(String::from).collect()) }
    pub fn first(&self) -> &str { self.0.first().map(|s| s.as_str()).unwrap_or("") }
    /// Resolve the field value from a JSON object.
    pub fn resolve(&self, entity: &serde_json::Value) -> crate::types::BqlValue {
        let mut current = entity;
        for part in &self.0 {
            match current.get(part) {
                Some(v) => current = v,
                None => return crate::types::BqlValue::Null,
            }
        }
        crate::types::BqlValue::from(current)
    }
}

/// A BQL comparison or boolean expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BqlExpr {
    /// field = value
    Eq(FieldPath, BqlValue),
    /// field != value
    Neq(FieldPath, BqlValue),
    /// field > value
    Gt(FieldPath, BqlValue),
    /// field < value
    Lt(FieldPath, BqlValue),
    /// field >= value
    Gte(FieldPath, BqlValue),
    /// field <= value
    Lte(FieldPath, BqlValue),
    /// field CONTAINS value
    Contains(FieldPath, BqlValue),
    /// expr AND expr
    And(Box<BqlExpr>, Box<BqlExpr>),
    /// expr OR expr
    Or(Box<BqlExpr>, Box<BqlExpr>),
    /// NOT expr
    Not(Box<BqlExpr>),
    /// Always true (no WHERE clause)
    True,
}

impl BqlExpr {
    /// Evaluate this expression against a single entity JSON object.
    pub fn evaluate(&self, entity: &serde_json::Value) -> bool {
        match self {
            Self::True => true,
            Self::Eq(f, v) => f.resolve(entity) == *v,
            Self::Neq(f, v) => f.resolve(entity) != *v,
            Self::Gt(f, v) => f.resolve(entity).cmp_for_order(v) == std::cmp::Ordering::Greater,
            Self::Lt(f, v) => f.resolve(entity).cmp_for_order(v) == std::cmp::Ordering::Less,
            Self::Gte(f, v) => matches!(f.resolve(entity).cmp_for_order(v), std::cmp::Ordering::Greater | std::cmp::Ordering::Equal),
            Self::Lte(f, v) => matches!(f.resolve(entity).cmp_for_order(v), std::cmp::Ordering::Less | std::cmp::Ordering::Equal),
            Self::Contains(f, v) => {
                if let (crate::types::BqlValue::Str(haystack), crate::types::BqlValue::Str(needle)) = (f.resolve(entity), v) {
                    haystack.contains(needle.as_str())
                } else { false }
            }
            Self::And(a, b) => a.evaluate(entity) && b.evaluate(entity),
            Self::Or(a, b) => a.evaluate(entity) || b.evaluate(entity),
            Self::Not(e) => !e.evaluate(entity),
        }
    }
}

/// A complete BQL query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BqlQuery {
    /// Entity type (FROM clause, e.g. "tasks").
    pub entity_type: EntityType,
    /// WHERE clause (BqlExpr::True if no filter).
    pub filter: BqlExpr,
    /// ORDER BY field + direction.
    pub order_by: Option<(FieldPath, OrderDir)>,
    /// LIMIT (None = no limit).
    pub limit: Option<usize>,
    /// OFFSET for pagination.
    pub offset: usize,
}

impl BqlQuery {
    pub fn all(entity_type: &str) -> Self {
        Self { entity_type: EntityType::new(entity_type), filter: BqlExpr::True, order_by: None, limit: None, offset: 0 }
    }
    pub fn with_filter(mut self, f: BqlExpr) -> Self { self.filter = f; self }
    pub fn order(mut self, field: &str, dir: OrderDir) -> Self { self.order_by = Some((FieldPath::new(field), dir)); self }
    pub fn limit(mut self, n: usize) -> Self { self.limit = Some(n); self }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entity(status: &str, priority: i64) -> serde_json::Value {
        serde_json::json!({"status": status, "priority": priority})
    }
    #[test] fn eq_match() { let e = BqlExpr::Eq(FieldPath::new("status"), BqlValue::Str("blocked".into())); assert!(e.evaluate(&entity("blocked", 3))); assert!(!e.evaluate(&entity("done", 3))); }
    #[test] fn and_expr() { let e = BqlExpr::And(Box::new(BqlExpr::Eq(FieldPath::new("status"), BqlValue::Str("blocked".into()))), Box::new(BqlExpr::Gt(FieldPath::new("priority"), BqlValue::Int(2)))); assert!(e.evaluate(&entity("blocked", 3))); assert!(!e.evaluate(&entity("blocked", 1))); }
    #[test] fn nested_path() { let entity = serde_json::json!({"dependency": {"depth": 5}}); let f = FieldPath::new("dependency.depth"); assert_eq!(f.resolve(&entity), BqlValue::Int(5)); }
    #[test] fn query_all() { let q = BqlQuery::all("tasks"); assert!(matches!(q.filter, BqlExpr::True)); }
}