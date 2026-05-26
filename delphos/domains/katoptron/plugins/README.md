# bkg-plugins

**Plugin discovery. YAML manifest. UI slot contributions.**

Plugins extend BKG with new UI views, prompt contributions, and runtime behaviors.
Dynamic loading is sandboxed via `bkg-vm`.

## Key Types

| Type | Purpose |
|---|---|
| `PluginManifest` | YAML manifest: id, version, name, ui_slots, prompt_contributions |
| `PluginRegistry` | Registered plugins + enabled/disabled state |
| `PluginLoader` | Validates manifests before registration |
| `UiSlot` | Component URL for a specific slot |
| `PromptContribution` | System prompt fragment injection |
