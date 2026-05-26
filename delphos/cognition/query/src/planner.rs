// planner.rs — query plan optimization.
// Deterministic: same query always produces same plan.
use crate::ast::BqlQuery;

/// A query execution plan (currently always TableScan — future: index-aware).
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub entity_type: String,
    pub estimated_cost: usize,
    pub strategy: ScanStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStrategy {
    /// Scan all entities and filter. O(n).
    TableScan,
    /// Future: use a projection index for O(log n) or O(1).
    IndexScan { index_name: String },
}

pub struct QueryPlanner;

impl QueryPlanner {
    pub fn plan(query: &BqlQuery, hint_entity_count: usize) -> QueryPlan {
        // Simple heuristic: always TableScan for now.
        // When bkg-projection indexes are available, this will pick IndexScan.
        QueryPlan {
            entity_type: query.entity_type.0.clone(),
            estimated_cost: hint_entity_count,
            strategy: ScanStrategy::TableScan,
        }
    }
}