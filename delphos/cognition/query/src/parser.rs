// parser.rs — minimal BQL string parser.
// Handles the most common query patterns. Deterministic, no regex.
use crate::ast::{BqlExpr, BqlQuery, FieldPath, OrderDir};
use crate::types::BqlValue;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("expected SELECT, got: {0}")]
    NoSelect(String),
    #[error("expected entity type after SELECT, got: {0}")]
    NoEntityType(String),
    #[error("unrecognized token: {0}")]
    UnrecognizedToken(String),
    #[error("incomplete expression: {0}")]
    Incomplete(String),
}

/// Parse a simple BQL query string.
/// Grammar (subset):
///   SELECT <entity_type>
///   [WHERE <field> <op> <value> [AND <field> <op> <value>]*]
///   [ORDER BY <field> [ASC|DESC]]
///   [LIMIT <n>]
pub fn parse(input: &str) -> Result<BqlQuery, ParseError> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() { return Err(ParseError::NoSelect("empty input".into())); }

    let mut idx = 0;
    // SELECT
    if tokens[idx].to_uppercase() != "SELECT" { return Err(ParseError::NoSelect(tokens[0].into())); }
    idx += 1;
    // entity_type
    if idx >= tokens.len() { return Err(ParseError::NoEntityType("end of input".into())); }
    let entity_type = tokens[idx].to_lowercase();
    idx += 1;

    let mut filter = BqlExpr::True;
    let mut order_by = None;
    let mut limit = None;

    while idx < tokens.len() {
        match tokens[idx].to_uppercase().as_str() {
            "WHERE" => {
                idx += 1;
                let (expr, consumed) = parse_expr(&tokens[idx..])?;
                filter = expr;
                idx += consumed;
            }
            "ORDER" => {
                idx += 1; // BY
                if idx < tokens.len() && tokens[idx].to_uppercase() == "BY" { idx += 1; }
                if idx >= tokens.len() { return Err(ParseError::Incomplete("ORDER BY needs field".into())); }
                let field = FieldPath::new(tokens[idx]); idx += 1;
                let dir = if idx < tokens.len() && tokens[idx].to_uppercase() == "DESC" { idx += 1; OrderDir::Desc } else { if idx < tokens.len() && tokens[idx].to_uppercase() == "ASC" { idx += 1; } OrderDir::Asc };
                order_by = Some((field, dir));
            }
            "LIMIT" => {
                idx += 1;
                if idx >= tokens.len() { return Err(ParseError::Incomplete("LIMIT needs number".into())); }
                limit = tokens[idx].parse::<usize>().ok();
                idx += 1;
            }
            other => return Err(ParseError::UnrecognizedToken(other.into())),
        }
    }

    let mut q = BqlQuery::all(&entity_type).with_filter(filter);
    if let Some((f, d)) = order_by { q = q.order(f.0.join(".").as_str(), d); }
    if let Some(n) = limit { q = q.limit(n); }
    Ok(q)
}

/// Parse one expression (possibly connected by AND/OR).
/// Returns (expr, tokens_consumed).
fn parse_expr(tokens: &[&str]) -> Result<(BqlExpr, usize), ParseError> {
    if tokens.len() < 3 { return Err(ParseError::Incomplete(format!("expr needs field op value, got: {:?}", tokens))); }
    let field = FieldPath::new(tokens[0]);
    let op = tokens[1];
    let raw_val = tokens[2].trim_matches('"');
    let value = parse_value(raw_val);
    let mut consumed = 3;

    let lhs = match op {
        "="  | "==" => BqlExpr::Eq(field, value),
        "!=" | "<>" => BqlExpr::Neq(field, value),
        ">"  => BqlExpr::Gt(field, value),
        "<"  => BqlExpr::Lt(field, value),
        ">=" => BqlExpr::Gte(field, value),
        "<=" => BqlExpr::Lte(field, value),
        "CONTAINS" | "contains" => BqlExpr::Contains(field, value),
        other => return Err(ParseError::UnrecognizedToken(other.into())),
    };

    // Check for AND/OR connector
    if consumed < tokens.len() {
        match tokens[consumed].to_uppercase().as_str() {
            "AND" => {
                consumed += 1;
                let (rhs, rhs_consumed) = parse_expr(&tokens[consumed..])?;
                consumed += rhs_consumed;
                return Ok((BqlExpr::And(Box::new(lhs), Box::new(rhs)), consumed));
            }
            "OR" => {
                consumed += 1;
                let (rhs, rhs_consumed) = parse_expr(&tokens[consumed..])?;
                consumed += rhs_consumed;
                return Ok((BqlExpr::Or(Box::new(lhs), Box::new(rhs)), consumed));
            }
            _ => {}
        }
    }
    Ok((lhs, consumed))
}

fn parse_value(s: &str) -> BqlValue {
    if s == "null" { return BqlValue::Null; }
    if s == "true" { return BqlValue::Bool(true); }
    if s == "false" { return BqlValue::Bool(false); }
    if let Ok(n) = s.parse::<i64>() { return BqlValue::Int(n); }
    if let Ok(f) = s.parse::<f64>() { return BqlValue::Float(f); }
    BqlValue::Str(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BqlExpr;
    #[test] fn parse_select_all() { let q = parse("SELECT tasks").unwrap(); assert_eq!(q.entity_type.0, "tasks"); assert!(matches!(q.filter, BqlExpr::True)); }
    #[test] fn parse_where_eq() { let q = parse(r#"SELECT tasks WHERE status = "blocked""#).unwrap(); assert!(matches!(q.filter, BqlExpr::Eq(..))); }
    #[test] fn parse_order_limit() { let q = parse("SELECT tasks ORDER BY priority DESC LIMIT 5").unwrap(); assert_eq!(q.limit, Some(5)); assert!(q.order_by.is_some()); }
    #[test] fn parse_and_expr() { let q = parse(r#"SELECT tasks WHERE status = "blocked" AND priority > 3"#).unwrap(); assert!(matches!(q.filter, BqlExpr::And(..))); }
    #[test] fn parse_no_select_fails() { assert!(parse("FROM tasks").is_err()); }
}