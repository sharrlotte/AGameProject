## REMOVED Requirements

### Requirement: Each AI behavior is a distinct component

**Reason**: Replaced by one `AiBehavior` runtime enum plus per-ship profiles; marker components made rule authoring impossible without Rust changes.

**Migration**: Use `ai-profiles`; current behavior lives in `AiRuntime.current`.

### Requirement: Behavior implies exactly one marker at a time

**Reason**: There are no behavior markers; exactly one current behavior is stored in `AiRuntime`.

**Migration**: Query `AiRuntime.current` instead of marker components.

### Requirement: AIShipController deleted

**Reason**: The controller was removed; `AiProfile` and `AiRuntime` replace it.

**Migration**: Use the `ai-profiles` runtime and profile types.

### Requirement: docs/ai.md reflects the component FSM

**Reason**: The document must describe profiles and prioritized rules, not a marker FSM.

**Migration**: Update `docs/ai.md` with the `ai-profiles` semantics.
