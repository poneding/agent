# MX02-05 Desktop Product Smoke and Documentation Spec

## Purpose

Add deterministic smoke coverage and user-facing documentation for the productized Desktop workspace so future changes can prove the real Desktop coding-agent flow without provider credentials.

## Product Context

M60 added a Desktop daily-driver smoke. MX02-05 updates that smoke to cover the productized shell, thread/composer workflow, provider/model selection, inspector panels, settings, and secret redaction.

## Scope

This spec delivers:

- Desktop product smoke fixture covering open workspace, create thread, choose provider/model, submit prompt, receive typed frames, show approval, show diff, show task progress, open settings, and verify no secret leakage
- `docs/desktop/desktop-productization.md` user workflow document
- `cargo xtask desktop-test` updates or explicit documented smoke command if no xtask change is required
- README, roadmap, milestone status updates at MX02 closeout

Owned by other specs:

- crash/restart recovery smoke remains in M62
- full release packaging smoke remains in M65

## Architecture

Smoke fixtures use deterministic typed Desktop/bridge data. They do not require live provider credentials, network, or real command execution. Docs describe product usage and verification commands without exposing secrets.

## File Layout

```text
docs/desktop/desktop-productization.md
apps/desktop/src/__tests__/desktop-smoke.test.tsx
xtask/
README.md
docs/roadmap/PRODUCT_ROADMAP.md
docs/milestones/README.md
```

## Public Contracts

No new runtime contract is required unless the smoke identifies missing typed snapshots. Any added snapshots must be serde-friendly, bounded, and redacted.

## State and Failure Handling

- Smoke uses stable ids, timestamps, and fixture ordering.
- Failed smoke points to the missing product affordance or secret leak.
- Docs state the smoke command and expected credential-free behavior.

## Security and Privacy

- Fixtures must include secret-like canaries and prove they render as redacted values.
- No raw API key, token, env value, or unbounded command output may appear in screenshots, fixtures, logs, or docs.

## Testing Strategy

- Vitest Desktop smoke for productized shell workflow
- `npm --prefix apps/desktop test`
- `npm --prefix apps/desktop run build`
- `cargo xtask desktop-test` if updated or documented as the Desktop product gate
- final workspace gates at closeout

## Acceptance Criteria

- Smoke proves the productized Desktop first screen and primary coding flow.
- Docs explain the Desktop workspace, command palette, thread/composer, inspector, settings, and smoke command.
- No secret leakage is present in fixtures or snapshots.

## Completion Signal

MX02-05 is complete when Desktop product smoke, docs, README/roadmap/milestone updates, `cargo xtask desktop-test`, final workspace gates, and closeout commit are complete.
