// executor.rs — BQL query executor. Runs queries against RealmState entities.
// Deterministic: same query + same state = same result, always.
use serde::{Deserialize, Serialize};
use crate::ast::{BqlQuery, OrderDir};

/// Result of a BQL query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub entity_type: String,
    pub rows: Vec<serde_json::Value>,
    pub total_scanned: usize,
    pub total_matched: usize,
}

impl QueryResult {
    pub fn empty(entity_type: &str) -> Self { Self { entity_type: entity_type.into(), rows: vec![], total_scanned: 0, total_matched: 0 } }
    pub fn len(&self) -> usize { self.rows.len() }
    pub fn is_empty(&self) -> bool { self.rows.is_empty() }
}

/// Executes BQL queries against a collection of JSON entities.
pub struct QueryExecutor;

impl QueryExecutor {
    pub fn new() -> Self { Self }

    /// Execute a BQL query against a flat list of entity JSON objects.
    /// Deterministic: iterates in BTreeMap order, sorts with stable sort.
    pub fn execute(&self, query: &BqlQuery, entities: &[serde_json::Value]) -> QueryResult {
        let total_scanned = entities.len();

        // 1. Filter
        let mut matched: Vec<&serde_json::Value> = entities.iter()
            .filter(|e| query.filter.evaluate(e))
            .collect();

        let total_matched = matched.len();

        // 2. Sort (stable — deterministic tie-breaking by entity order)
        if let Some((field, dir)) = &query.order_by {
            matched.sort_by(|a, b| {
                let va = field.resolve(a);
                let vb = field.resolve(b);
                let ord = va.cmp_for_order(&vb);
                match dir { OrderDir::Asc => ord, OrderDir::Desc => ord.reverse() }
            });
        }

        // 3. Offset + Limit
        let rows: Vec<serde_json::Value> = matched.iter()
            .skip(query.offset)
            .take(query.limit.unwrap_or(usize::MAX))
            .map(|v| (*v).clone())
            .collect();

        QueryResult { entity_type: query.entity_type.0.clone(), rows, total_scanned, total_matched }
    }
}

impl Default for QueryExecutor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BqlExpr, BqlQuery, FieldPath};
    use crate::types::BqlValue;

    fn tasks() -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({"id":"T-1","status":"blocked","priority":3}),
            serde_json::json!({"id":"T-2","status":"done","priority":1}),
            serde_json::json!({"id":"T-3","status":"blocked","priority":5}),
            serde_json::json!({"id":"T-4","status":"in-progress","priority":2}),
        ]
    }

    #[test] fn filter_by_status() {
        let q = BqlQuery::all("tasks").with_filter(BqlExpr::Eq(FieldPath::new("status"), BqlValue::Str("blocked".into())));
        let r = QueryExecutor::new().execute(&q, &tasks());
        assert_eq!(r.total_matched, 2);
        assert_eq!(r.total_scanned, 4);
    }
    #[test] fn order_by_priority_desc() {
        let q = BqlQuery::all("tasks").with_filter(BqlExpr::Eq(FieldPath::new("status"), BqlValue::Str("blocked".into()))).order("priority", OrderDir::Desc);
        let r = QueryExecutor::new().execute(&q, &tasks());
        assert_eq!(r.rows[0]["priority"], 5);
        assert_eq!(r.rows[1]["priority"], 3);
    }
    #[test] fn limit_and_offset() {
        let q = BqlQuery::all("tasks").limit(2);
        let r = QueryExecutor::new().execute(&q, &tasks());
        assert_eq!(r.len(), 2);
    }
    #[test] fn empty_result() {
        let q = BqlQuery::all("tasks").with_filter(BqlExpr::Eq(FieldPath::new("status"), BqlValue::Str("nonexistent".into())));
        let r = QueryExecutor::new().execute(&q, &tasks());
        assert!(r.is_empty());
    }
    #[test] fn and_filter() {
        let expr = BqlExpr::And(Box::new(BqlExpr::Eq(FieldPath::new("status"), BqlValue::Str("blocked".into()))), Box::new(BqlExpr::Gt(FieldPath::new("priority"), BqlValue::Int(3))));
        let r = QueryExecutor::new().execute(&BqlQuery::all("tasks").with_filter(expr), &tasks());
        assert_eq!(r.total_matched, 1); // only T-3 with priority=5
        assert_eq!(r.rows[0]["id"], "T-3");
    }
}