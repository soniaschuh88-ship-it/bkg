//! bkg-eval — task evaluation scorecards + evidence.
//! Single source of truth for all quality scoring in DELPHOS.
pub mod batch; pub mod evidence; pub mod scorecard;
pub use evidence::{EvalEvidence, EvalSignal};
pub use scorecard::{EvalResult, EvalScore, Scorecard};
pub use batch::EvalBatch;
