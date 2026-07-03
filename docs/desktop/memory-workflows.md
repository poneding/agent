# Desktop Memory Workflows

M64 adds a Desktop-safe memory surface in the Tasks route. Desktop renders `memory_snapshot` bridge frames produced from shared runtime and memory contracts. It does not implement memory retrieval, redaction, proposal eligibility, retention, tombstone, import/export, or rebuild policy.

## Where It Appears

Open the Tasks view. When the conversation state has a memory snapshot, Desktop shows a Memory panel near the task and workflow surfaces.

The panel shows:

- memory enabled/disabled state
- active record counts by scope and sensitivity-safe summaries
- pending, committed, rejected, and expired proposal counts
- retrieval preview counts for selected and rejected candidates
- runtime influence trace counts and reasons
- privacy diagnostics and degraded states
- retention controls, tombstone count, import/export state, and rebuild evidence when present

If no memory snapshot has arrived, Desktop should show an empty or unavailable state instead of deriving memory state from local UI data.

## Records

Record rows are bounded summaries from `agent-memory` product contracts. They can show ids, scope-safe metadata, confidence, sensitivity labels, source descriptions, and redacted previews.

Sensitive record text is not shown in default Desktop summaries. If a record preview is `[redacted]`, Desktop should preserve that exact state and avoid adding its own reveal behavior unless a later runtime contract introduces one.

## Retrieval And Influence

Retrieval preview and runtime influence are separate concepts:

- retrieval preview explains candidate selection and rejection for a query
- influence explains which memories were actually inserted into runtime context for a turn

Desktop can display selected and rejected counts, ranks, reasons, matched terms, bounded previews, and influence diagnostics only from bridge-safe fields. It must not rerank candidates or infer why a memory was selected.

## Proposals

Memory proposals represent candidate writes that need explicit review. Desktop can show proposal id, status, sensitivity, confidence, review eligibility, source-event count, summary, and redacted resolution reason.

Default proposal summaries do not include raw proposal content. Approve, reject, expire, or commit controls should only be enabled when a typed bridge command exists for the action and the proposal summary says the action is reviewable.

## Disabled Memory

When memory is disabled, Desktop should show the runtime diagnostic and avoid presenting selected influence for the turn. This is a normal product state, not a UI error.

Common disabled or degraded states:

- memory disabled by config
- memory disabled by policy
- memory unavailable during startup or recovery
- snapshot missing because the bridge has not emitted one yet
- retrieval unavailable because required memory indexes are stale

Desktop should render the provided diagnostic and direct users to settings or CLI diagnostics only when the bridge supplies that next action.

## Retention, Tombstones, Import, Export, And Rebuild

Retention controls describe how memory records are kept and summarized. Tombstones represent deletion evidence and prevent deleted records from silently reappearing during import or rebuild.

Desktop can show:

- tombstone counts
- delete evidence summaries
- export record counts
- import accepted, skipped, rejected, and tombstone counts
- rebuild indexed records, skipped tombstones, and stale embedding removals

Default export excludes sensitive records. Desktop should not expose raw archive payloads, embeddings, or unbounded import errors.

## Privacy And Redaction

Memory snapshots are bounded and redacted before reaching Desktop. The UI should preserve those fields directly and avoid displaying:

- raw sensitive memory text
- raw proposal content
- embeddings
- raw prompts
- unbounded command logs
- raw API keys, tokens, passwords, or secret-like values

If a field contains a redaction marker, Desktop should display it as evidence that privacy controls acted. It should not attempt to reconstruct hidden content.

## Smoke Evidence

M64 closeout adds `default-m64-memory-smoke` to the default eval suite. Run:

```bash
cargo xtask eval
npm --prefix apps/desktop test -- --run src/__tests__/memory-ui.test.tsx
```

The eval covers deterministic retrieval, influence, proposal review, import/export, delete/tombstone, rebuild, disabled-memory diagnostics, and redaction. The Desktop test covers rendering of the memory panel from bridge-safe snapshots.
