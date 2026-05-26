# bkg-cli

**The `bkg` command-line binary.**

Single entry point for all BKG operations.

## Commands

```
bkg init               — initialize a BKG project
bkg providers          — manage LLM providers
bkg chat               — interactive chat
bkg agent              — agent management
bkg task               — task management
bkg mission            — mission management
bkg session            — session management
bkg snapshot           — snapshot management
bkg replay             — replay events
```

## Invariant

CLI commands never mutate state directly.
All mutations go through `EventPipeline` → `Realm::submit_event()`.
