// lints.rs — Workspace lint rules for DELPHOS.
//
// These lints are documented here and enforced in Cargo.toml [lints.rust].
// Every crate in the workspace inherits them.
//
// This module documents WHY each lint exists and what it prevents.
// Single source of truth for workspace code quality policy.

/// Documents the workspace-level lint policy.
///
/// Enforced in [workspace.lints.rust] in the root Cargo.toml.
/// All crates inherit these automatically.
pub struct WorkspaceLints;

impl WorkspaceLints {
    /// Returns the lint policy as a human-readable description.
    /// In production this would be parsed and validated at build time.
    pub fn policy() -> &'static [LintRule] { RULES }
}

#[derive(Debug, Clone, Copy)]
pub struct LintRule {
    pub name: &'static str,
    pub level: LintLevel,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintLevel { Warn, Deny, Forbid }

impl std::fmt::Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Warn => f.write_str("warn"), Self::Deny => f.write_str("deny"), Self::Forbid => f.write_str("forbid") }
    }
}

static RULES: &[LintRule] = &[
    LintRule {
        name: "unsafe_code",
        level: LintLevel::Forbid,
        reason: "unsafe code breaks the determinism guarantees. No exceptions.",
    },
    LintRule {
        name: "unused",
        level: LintLevel::Deny,
        reason: "unused items are dead code and increase cognitive overhead.",
    },
    LintRule {
        name: "dead_code",
        level: LintLevel::Deny,
        reason: "dead code cannot be tested and drifts from the real system.",
    },
    LintRule {
        name: "clippy::panic",
        level: LintLevel::Warn,
        reason: "panics are non-deterministic across platforms. Use Result.",
    },
    LintRule {
        name: "clippy::unwrap_used",
        level: LintLevel::Warn,
        reason: "unwrap() can panic. Use expect() with context or proper error handling.",
    },
    LintRule {
        name: "clippy::expect_used",
        level: LintLevel::Warn,
        reason: "expect() can panic. Prefer ? propagation in production paths.",
    },
    LintRule {
        name: "clippy::todo",
        level: LintLevel::Warn,
        reason: "todo!() is a deferred panic. Track in TASKS.md instead.",
    },
    LintRule {
        name: "clippy::dbg_macro",
        level: LintLevel::Deny,
        reason: "dbg!() has side effects and must not appear in committed code.",
    },
    LintRule {
        name: "clippy::print_stdout",
        level: LintLevel::Deny,
        reason: "use structured logging (tracing) not println! in library code.",
    },
    LintRule {
        name: "clippy::needless_pass_by_value",
        level: LintLevel::Warn,
        reason: "unnecessary ownership prevents efficient zero-copy patterns.",
    },
];

impl WorkspaceLints {
    /// Validate that no forbidden patterns appear in a code snippet.
    /// Used in tests to verify policy compliance.
    pub fn check_snippet(code: &str) -> Vec<&'static LintRule> {
        let mut violations = Vec::new();
        for rule in RULES {
            let pattern = match rule.name {
                "unsafe_code" => "unsafe ",
                "clippy::dbg_macro" => "dbg!(",
                "clippy::print_stdout" => "println!(",
                "clippy::todo" => "todo!()",
                _ => continue,
            };
            if code.contains(pattern) && rule.level == LintLevel::Forbid || rule.level == LintLevel::Deny {
                violations.push(rule);
            }
        }
        violations
    }

    /// Generate the Cargo.toml [workspace.lints.rust] section.
    pub fn cargo_toml_section() -> String {
        let mut s = "[workspace.lints.rust]\n".to_string();
        for rule in RULES {
            if !rule.name.starts_with("clippy::") {
                s.push_str(&format!("{} = \"{}\"\n", rule.name, rule.level));
            }
        }
        s.push_str("\n[workspace.lints.clippy]\n");
        for rule in RULES {
            if let Some(name) = rule.name.strip_prefix("clippy::") {
                s.push_str(&format!("{name} = \"{}\"\n", rule.level));
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn policy_not_empty() { assert!(!WorkspaceLints::policy().is_empty()); }
    #[test] fn unsafe_is_forbidden() {
        let unsafe_rule = RULES.iter().find(|r| r.name == "unsafe_code").unwrap();
        assert_eq!(unsafe_rule.level, LintLevel::Forbid);
    }
    #[test] fn cargo_section_contains_unsafe() {
        let s = WorkspaceLints::cargo_toml_section();
        assert!(s.contains("unsafe_code"));
    }
    #[test] fn snippet_detects_println() {
        let violations = WorkspaceLints::check_snippet("println!(\"hello\");");
        assert!(!violations.is_empty());
    }
}
