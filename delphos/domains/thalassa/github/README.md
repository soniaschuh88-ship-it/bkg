# bkg-github

**GitHub integration. Issue import. PR creation. OAuth.**

Connects the BKG task system to GitHub issues and pull requests.

## Key Types

| Type | Purpose |
|---|---|
| `GithubAuth` | `{ owner, repo, token }` |
| `GithubToken` | OAuth token with expiry check |
| `GithubIssue` | Issue data + `to_task_title()` |
| `IssueImport` | Label/state filter + import URL |
| `PullRequest` | `{ head, base, strategy: Squash/Merge/Rebase }` |
