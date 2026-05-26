//! bkg-github — GitHub integration: issue import, PR creation, OAuth.
//! Single source of truth for all GitHub connectivity in DELPHOS.
pub mod auth; pub mod issue; pub mod pr;
pub use auth::{GithubAuth, GithubToken};
pub use issue::{GithubIssue, IssueImport};
pub use pr::{PullRequest, PrStrategy};
