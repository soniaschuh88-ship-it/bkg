# bkg-project

**Project registry. Scoped configuration. 5 model lanes per project.**

Projects are the unit of isolation in BKG. All data is project-scoped.
No data leaks between projects.

## Key Types

| Type | Purpose |
|---|---|
| `ProjectRegistry` | `~/.bkg/bkg-central.db` (sled-backed) |
| `Project` | `{ id, title, path, settings, created_at }` |
| `ProjectSettings` | Model lanes, provider config, workflow settings |

## Model Lanes

Each project has 5 configurable model lanes:
`fast`, `balanced`, `powerful`, `review`, `embedding`
