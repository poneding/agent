# M17-04 CLI Streaming Output Spec

## Purpose

Render model deltas incrementally in CLI text and JSON modes while preserving script-safe output.

## Product Context

Once providers expose real streams, users should see responses as they arrive. JSON clients also need incremental frames for TUI/Desktop bridge reuse.

## Scope

This slice delivers:

- configured turn execution path that exposes runtime events incrementally
- text mode printing `ModelDelta` text as it arrives
- JSON mode emitting NDJSON event frames as events arrive
- final done frame after stream completion
- deterministic delayed-stream fixture tests

Out of scope:

- full-screen TUI rendering
- terminal raw mode
- partial tool-call UI

## Architecture

`agent-core` should expose a streaming configured turn API that yields `RuntimeEvent` values. `agent-cli` consumes that event stream and renders either text deltas or `UiJsonFrame::Event` lines. Existing collected APIs may remain as compatibility wrappers that collect the streaming API.

## State And Failure Handling

- `Started`, `Delta`, and `Completed` events are emitted as soon as consumed.
- On stream error, JSON emits an error frame followed by done.
- Text mode writes any completed deltas and then returns non-zero on terminal errors.
- Output flushing is explicit in binary code, but pure tests assert event order.

## Security And Privacy

JSON frames use `agent-ui-bridge` normalization. Text mode receives only model deltas after prompt/user secret redaction already applied by the runtime path.

## Testing Strategy

- delayed fake stream proves deltas are observed in order
- JSON prompt mode emits model delta frames before final response
- text prompt mode prints streamed deltas without NDJSON decorations
- error stream emits redacted error frames

## Acceptance Criteria

- CLI prompt output no longer waits for the whole model result internally.
- NDJSON remains parseable and one frame per line.
- Existing batch tests remain green.

## Completion Signal

M17-04 is complete when CLI streaming tests and workspace gates pass.
