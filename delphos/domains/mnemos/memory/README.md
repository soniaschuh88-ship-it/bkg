# bkg-memory

**Causal memory graph. Knowledge synthesis.**

Stores semantic memories as nodes in a causal graph.
Each memory has provenance (which events caused it).

## Key Types

| Type | Purpose |
|---|---|
| `MemoryGraph` | Causal graph of knowledge nodes |
| `MemoryNode` | `{ id, content, provenance, importance }` |
| `MemoryEdge` | `{ from, to, relation_type }` |
