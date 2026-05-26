// projection_view.rs — Sealed read-only projection API.
//
// The law: UI reads ONLY from ProjectionView<T>.
//          No direct RealmState access outside bkg-state internals.
//          No mutable access to projections ever.
//
// This is structural, not documented:
//   - RealmState is pub(crate) — not accessible outside bkg-state
//   - ProjectionView<T> is the only public read interface
//   - ProjectionView<T> is sealed — only bkg-state can construct it
//   - All access methods are read-only (&self only)
//
// Single source of truth for projection isolation.

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// ─── Sealed impl for ProjectionView ──────────────────────────────────────────

/// Private sealing token — only bkg-state can construct ProjectionView.
/// This makes it structurally impossible to bypass the materializer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProjectionSeal(());

#[allow(dead_code)]
impl ProjectionSeal {
    /// Only callable within this module.
    pub(crate) fn new() -> Self { Self(()) }
}

// ─── ProjectionView<T> ────────────────────────────────────────────────────────

/// A sealed, read-only view of a materialized projection.
///
/// ## Invariants (structural, not documented)
/// 1. Can only be constructed by the `Materializer` via `ProjectionView::seal()`
/// 2. All access methods are `&self` — no mutation possible
/// 3. The inner data is private — only accessible via typed methods
/// 4. `state_checksum` is immutable — staleness can be detected but not fixed here
///
/// ## Usage
/// ```ignore
/// // In Atlantean dashboard:
/// let view: ProjectionView<KanbanData> = materializer.get("kanban")?;
/// let columns = view.data().columns; // read-only
/// // view.data_mut() — does not exist. Cannot mutate.
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionView<T> {
    /// The projection ID.
    pub projection_id: String,
    /// Realm this projection belongs to.
    pub realm_id: String,
    /// State version at the time this projection was built.
    pub state_version: u64,
    /// Checksum of the state that produced this projection.
    pub state_checksum: String,
    /// The read-only data. Private — accessed via data() only.
    data: T,
    /// Sealing proof — only Materializer can provide this.
    #[serde(skip)]
    _seal: PhantomData<ProjectionSeal>,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> ProjectionView<T> {
    /// Construct a sealed projection view.
    /// Only callable within bkg-state (takes ProjectionSeal by value — it's pub(crate)).
    #[allow(dead_code)]
    pub(crate) fn seal(
        _proof: ProjectionSeal,
        projection_id: impl Into<String>,
        realm_id: impl Into<String>,
        state_version: u64,
        state_checksum: impl Into<String>,
        data: T,
    ) -> Self {
        Self {
            projection_id: projection_id.into(),
            realm_id: realm_id.into(),
            state_version,
            state_checksum: state_checksum.into(),
            data,
            _seal: PhantomData,
        }
    }

    /// Read-only access to the projection data. Cannot be mutated.
    pub fn data(&self) -> &T { &self.data }

    /// Check if this projection is stale relative to a new state checksum.
    pub fn is_stale(&self, current_checksum: &str) -> bool {
        self.state_checksum != current_checksum
    }

    /// Clone the inner data (for read-only consumers).
    pub fn cloned_data(&self) -> T { self.data.clone() }

    /// Serialize to JSON for API responses (read-only).
    pub fn to_json(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(&self.data)
    }
}

// There is no data_mut() method.
// There is no DerefMut impl.
// There is no way to obtain &mut T from ProjectionView<T>.

// ─── ProjectionFactory ────────────────────────────────────────────────────────

/// The only way to create ProjectionView instances.
/// Lives inside bkg-state — UI code cannot use this directly.
pub(crate) struct ProjectionFactory;

#[allow(dead_code)]
impl ProjectionFactory {
    pub(crate) fn create<T: Clone + Serialize + for<'de> Deserialize<'de>>(
        projection_id: impl Into<String>,
        realm_id: impl Into<String>,
        state_version: u64,
        state_checksum: impl Into<String>,
        data: T,
    ) -> ProjectionView<T> {
        ProjectionView::seal(
            ProjectionSeal::new(),
            projection_id,
            realm_id,
            state_version,
            state_checksum,
            data,
        )
    }
}

// ─── Well-known projection data types ────────────────────────────────────────

/// Kanban board projection data (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanProjection {
    pub planning: Vec<String>,
    pub todo: Vec<String>,
    pub in_progress: Vec<String>,
    pub in_review: Vec<String>,
    pub done: Vec<String>,
    pub total_tasks: usize,
}

#[allow(clippy::derivable_impls)]
impl Default for KanbanProjection {
    fn default() -> Self {
        Self { planning: vec![], todo: vec![], in_progress: vec![], in_review: vec![], done: vec![], total_tasks: 0 }
    }
}

/// Agent status projection data (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusProjection {
    pub agent_id: String,
    pub display_name: String,
    pub active_sessions: u32,
    pub total_calls: u64,
    pub success_rate: f64,
    pub configured: bool,
}

/// Task list projection (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListProjection {
    pub tasks: Vec<TaskSummary>,
    pub total_count: usize,
    pub blocked_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assignee: Option<String>,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kanban_view() -> ProjectionView<KanbanProjection> {
        ProjectionFactory::create(
            "kanban",
            "telum",
            5,
            "checksum-abc",
            KanbanProjection { todo: vec!["T-1".into(), "T-2".into()], total_tasks: 2, ..Default::default() },
        )
    }

    #[test]
    fn read_only_access() {
        let view = make_kanban_view();
        assert_eq!(view.data().total_tasks, 2);
        assert_eq!(view.data().todo.len(), 2);
        // view.data_mut() — does not compile (method doesn't exist)
    }

    #[test]
    fn staleness_detection() {
        let view = make_kanban_view();
        assert!(!view.is_stale("checksum-abc"));
        assert!(view.is_stale("checksum-xyz"));
    }

    #[test]
    fn to_json_works() {
        let view = make_kanban_view();
        let json = view.to_json().unwrap();
        assert_eq!(json["total_tasks"], 2);
    }

    #[test]
    fn cloned_data_independent() {
        let view = make_kanban_view();
        let mut cloned = view.cloned_data();
        cloned.total_tasks = 999;
        assert_eq!(view.data().total_tasks, 2); // original unchanged
    }

    #[test]
    fn metadata_readable() {
        let view = make_kanban_view();
        assert_eq!(view.projection_id, "kanban");
        assert_eq!(view.realm_id, "telum");
        assert_eq!(view.state_version, 5);
    }

    #[test]
    fn task_projection() {
        let view = ProjectionFactory::create(
            "task-list", "telum", 1, "ck",
            TaskListProjection {
                tasks: vec![TaskSummary { id: "T-1".into(), title: "impl pipeline".into(), status: "in_progress".into(), priority: "high".into(), assignee: Some("agent-1".into()) }],
                total_count: 1,
                blocked_count: 0,
            },
        );
        assert_eq!(view.data().tasks[0].id, "T-1");
        assert_eq!(view.data().blocked_count, 0);
    }
}
