# bkg-agents

**7 agent types. Credential fallback chain. BkgSupervised mode.**

Agents are inhabitants of the BKG world. They do NOT control the world.
Every agent action goes through `EventPipeline`.

## 7 Agent Types

1. `Orchestrator` — decomposes goals into task DAGs
2. `Executor` — runs bash/file/grep tools
3. `Reviewer` — applies workflow review gates
4. `Researcher` — web search + knowledge synthesis
5. `Planner` — generates implementation plans
6. `Debugger` — identifies root causes
7. `Summarizer` — condenses long outputs

## Credential Fallback Chain

```
1. User's own key     (per-project config)
2. Admin global key   (~/.bkg/global-providers.json)
3. Env variable       (ANTHROPIC_API_KEY, etc.)
4. Anonymous tier     (Kilo + LLM7 — no key required)
```
