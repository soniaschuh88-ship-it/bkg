# bkg-atlantean

**Cyberpunk/Atlantis dashboard. The primary UI.**

Private mode: WebLLM in-browser (zero data leaves the machine).
Cloud mode: 13 free providers.

## Architecture

```
HTTP API (Actix-web) → bkg-providers + bkg-agents + bkg-session
WebSocket / SSE      → live updates from bkg-session
Static assets        → Cyberpunk/Atlantis themed UI
```

## Key Features

- Provider management (13 providers, free detection, toggles)
- Agent management (7 agents, live status probe)
- Session management (CRUD, SSE streaming)
- Inspector tab (real-time event inspection)
- Onboarding wizard
- Private/Cloud mode switch

## Routes

```
GET  /api/providers        — list providers
POST /api/chat             — send message
GET  /api/agents           — list agents
GET  /api/sessions         — list sessions
GET  /api/sessions/:id/sse — live stream
```
