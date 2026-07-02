# CLI Memory Review

M64 adds credential-free memory review commands for inspecting records, previewing retrieval influence, deciding proposed writes, importing/exporting archives, deleting records with tombstones, and rebuilding index evidence. The CLI renders shared `agent-memory` contracts; privacy, proposal, retrieval, and retention rules stay in the memory crate.

## Snapshot

```bash
agent-cli memory snapshot
agent-cli --json memory snapshot
```

The snapshot summarizes active records, pending proposals, retention controls, tombstone counts, and privacy diagnostics. JSON mode emits a `memory_snapshot` frame with a `MemoryProductSnapshot`.

## List And Inspect

```bash
agent-cli memory list
agent-cli memory inspect public-memory
agent-cli --json memory list
agent-cli --json memory inspect public-memory
```

Default summaries include ids, scope-safe metadata, sensitivity, confidence, source descriptions, and bounded previews. Sensitive record text is redacted.

## Retrieval Preview

```bash
agent-cli memory retrieve --query "fixture memory"
agent-cli memory retrieve --query "fixture memory" --max-selected 1
agent-cli --json memory retrieve --query "fixture memory" --max-selected 1
```

Retrieval preview shows selected and rejected candidate counts, ranks, reasons, matched terms, bounded previews, and influence evidence. JSON mode emits a `memory_retrieval_preview` frame with retrieval and influence summaries.

## Proposal Review

```bash
agent-cli memory proposal list
agent-cli memory proposal inspect proposal-public
agent-cli memory proposal approve proposal-public
agent-cli memory proposal reject proposal-sensitive --reason "not accurate"
agent-cli memory proposal expire proposal-sensitive --reason "stale"
agent-cli memory proposal commit proposal-public
```

Proposal summaries include status, sensitivity, confidence, review eligibility, source-event count, summary text, and redacted resolution reasons. Proposal content is not rendered in default text or JSON output.

## Import, Export, Delete, And Rebuild

```bash
agent-cli --json memory export
agent-cli --json memory import memory.json
agent-cli memory delete public-memory --reason "cleanup"
agent-cli memory rebuild-index
```

Export excludes sensitive records by default. Import reports accepted, skipped, rejected, and tombstone counts. Delete creates a tombstone with a redacted reason. Rebuild reports indexed records, skipped tombstones, and stale embedding removals.

## JSON Frame Types

- `memory_list`
- `memory_snapshot`
- `memory_inspect`
- `memory_retrieval_preview`
- `memory_proposal_list`
- `memory_proposal_inspect`
- `memory_proposal_decision`
- `memory_proposal_commit`
- `memory_deleted`
- `memory_export`
- `memory_import`
- `memory_rebuild`
- `memory_error`

## Privacy Guarantees

CLI memory output must not include raw sensitive memory text, proposal content, embeddings, or unbounded command logs. Secret-like values in ids, summaries, reasons, diagnostics, and errors are redacted before rendering.
