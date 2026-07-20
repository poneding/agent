# ruse-tui Redesign & Optimization Guide

Status: design guidance (not yet implemented). Author: TUI/harness review, synthesized
from a 10-agent audit of `crates/ruse-tui` plus competitive analysis of Claude Code and
OpenAI Codex CLIs and ratatui best-practice research.

This document is the reasoning and evidence behind the redesign. The companion file
[`tui-redesign-goal.md`](./tui-redesign-goal.md) is the self-contained prompt to start the
implementation session from.

---

## 1. What we're fixing (the user's complaint, decoded)

The user reported that the whole TUI interaction "feels bad", calling out three things by
name: the Approve confirmation flow, the position of `Working… (Ctrl+C to interrupt)`, and
on-screen information display. The audit confirms all three and traces them to a small set
of root causes rather than many independent bugs.

The good news up front: the **architecture is sound**. `TuiApp::reduce` is a pure state
machine returning `TuiEffect`s, side effects run outside it, the runtime hides behind two
narrow traits (`TerminalTurnRuntime`, `TerminalCommandRuntime`), and rendering is a pure
function of state (`render_app_to_string` + `TestBackend`). We are refining a good skeleton,
not rewriting.

---

## 2. Root-cause clusters (ranked)

### Cluster A — Approvals are not modal and the keymap is booby-trapped (CRITICAL, safety)

The approval subsystem has a clean state core but a dangerous interaction model:

- **Bare `Enter` grants the focused approval.** When the composer is empty and any request is
  focused, `Enter → ApprovalPanelAction::Allow` (`terminal_app.rs:428-431`). The first request
  auto-focuses (`approval_panel.rs:182-184`). So the single most reflexive key in the app
  silently approves a pending file-write or shell command, with no confirmation. This directly
  violates the app's own `default-deny` posture.
- **The approval controls are usually invisible.** For the two most common cases — a file edit
  or a command — the Auto side panel resolves `Diff > Command > Approval` (`render.rs:372-382`),
  and a file-edit approval derives a `diff_panel` on every frame (`app.rs:361-368`). So the user
  sees a diff with **zero** approval instructions, while `Enter/a/d/Esc` are silently live. The
  only hint string (`approval_panel.rs:109`) is exactly what gets suppressed.
- **Approvals never force themselves on screen.** An incoming request only mutates panel state;
  it does not raise a modal or steal focus. On any non-Auto panel (Trace/Tasks/Status) or below
  60 columns (where side panels aren't drawn at all, `render.rs:82-88`), the gate is **invisible
  but still armed** — a blind approve/deny on a safety-gated action.
- **`Esc` denies.** Everywhere else `Esc` means "close/back", but in the approval hijack it maps
  to Abort → Deny "aborted from TUI" (`command.rs:1274-1302`). The universal escape key
  irreversibly fails the tool call.
- **Draft text disables approval.** The keys are gated on `composer.text.is_empty()`. Type
  "looks good" then `Enter` and you *queue a prompt* instead of approving, with no explanation.
- Reasons are canned placeholders ("approved from TUI" / "denied from TUI"); no queue list for
  triaging multiple requests; the timed lease is granted implicitly by `Enter`.

### Cluster B — The live indicator is misplaced and starved (HIGH, the stated dislike)

- The `Working…` indicator is a **composer top-border title** (`render.rs:442-444`). It is
  cramped (a border row), silently truncates the `(Ctrl+C to interrupt)` hint on narrow
  terminals, is easy to miss, and has no room for elapsed time or token counts.
- The one always-present line of chrome — the bottom status bar (`render.rs:548-569`) — is
  **completely inert during a turn**: no elapsed, no tokens, no interrupt hint, no spinner. The
  user's eye lands there and finds no liveness.
- The spinner is driven by `spinner_tick`, incremented only on `TuiAction::Tick`, which only
  fires on poll timeout (`terminal_app.rs:327-329`). **Typing during a turn freezes the
  spinner** — the "live" signal dies exactly when the user is active.
- Both Claude Code and Codex converge on the same answer: a single dedicated line, directly
  above the composer, showing `spinner + verb + elapsed + tokens + interrupt-hint`, present only
  while working. That is the target.

### Cluster C — "Diagnostic" conflates errors, notes, and successes (HIGH, the noise complaint)

- `TuiEffect::LocalDiagnostic` is the catch-all for no-op guidance ("no running turn to cancel",
  "no diff available yet", etc.) **and** clipboard success ("copied via OSC52"), and every one
  renders as a **red error chip** (`render.rs:335,350`). Errors are visually indistinguishable
  from notes and successes — this is the primary driver of perceived noise, and it trains users
  to ignore red.
- Core state is missing from persistent chrome: `connection`, turn-state, `mode`, and
  `permission_state` live only in the opt-in Status panel — and `permission_state` renders
  **nowhere**. After the first turn the welcome screen (which showed connection) is gone.
- `connection` is churned from two out-of-sync sources that leak raw `Debug` strings like
  `"modelstreaming"` and raw codes like `"waiting_approval"` to the user (`app.rs:590/604` vs
  `status_panel.rs:146-193`).
- ~460 lines of `Progress`-note arms in `transcript.rs:234-698` are **dead code** (double-gated
  out), which makes the whole noise taxonomy impossible to reason about.

### Cluster D — The composer can't actually edit text (CRITICAL, table-stakes)

- **Multiline is unreachable.** `InsertNewline` is gated on a `multiline` flag that defaults
  false and is only flipped by `ToggleMultiline`, which **has no keybinding anywhere**. So
  `Shift+Enter` is dead and you can never type a second line.
- **Bracketed paste is never enabled** (`terminal.rs:55-63`). Pasted multi-line text arrives as
  key events; the first embedded newline is `Enter → Submit`, so pasting a stack trace fires a
  truncated turn. The wired `Event::Paste` handler is dead in practice.
- **No cursor movement.** Text is a `String` with push/pop-at-end only; `Left/Right/Home/End`
  are unbound for the composer, no word motions, no mid-line edit. Fixing a typo means
  backspacing everything after it.
- Long input is clipped with no wrap/scroll; `composer_cursor_position` only understands the
  first line (`render.rs:535-546`).
- `tui-textarea` **is already a dependency** (`Cargo.toml`) and unused. Adopting it fixes all of
  the above for free (grapheme-aware cursor, multiline, motions, paste).

### Cluster E — Event loop and turn-state architecture (MEDIUM, enables everything else)

- The loop calls `terminal.draw()` **unconditionally every ~50ms** and ticks the spinner even
  when idle: ~20 repaints/sec forever, burning battery. Gate draws on a dirty flag.
- Turn/activity truth has **no home**: it's split between the `waiting_activity()` transcript
  heuristic and a loose `turn_active` bool threaded (with `active_cancellation`, `queued_prompts`)
  through four `apply_terminal_effect*` overloads and a six-field struct. This derivation causes
  the concrete gaps: no spinner for queued/recovery turns until streaming starts, an unreachable
  `Waiting` state, and it blocks an accurate elapsed timer.
- `diff_panel`/`command_panel` are **rebuilt on every bridge frame** (`app.rs:359-368`), so
  scrolling a diff during streaming snaps back to top and the panel can vanish mid-read.
- `Ctrl+C` has no force-quit escape hatch when a turn hangs, and spams duplicate diagnostics.
- Detached thread-per-turn with no join and **untagged frames**: late frames from a cancelled
  turn can bleed into the next turn's transcript.

---

## 3. Competitive model (what Claude Code & Codex actually do)

Both converge on the same shape; adopt it.

- **Layout:** one scrolling transcript above a fixed bottom cluster — ephemeral activity line →
  persistent composer → one-line mode/hint footer. The interactive surface never moves while
  content streams.
- **Live line (the fix for the stated dislike):** one line, only while working:
  `✳ Simmering… (12s · ↑1.2k tokens · esc to interrupt)`. Two motion channels (spinner frame +
  rotating gerund verb) prove liveness better than a spinner alone. Interrupt hint lives on this
  line, coupled to actionability so it's never stale chrome.
- **Approvals = the safety spine:** a prompt that shows the concrete action (reuse the real
  diff/command widget, not a paraphrase), defaults to the **safe** option, and offers
  `Yes / Yes-and-don't-ask-again-scoped / No-and-tell-it-what-to-do-differently`. "Always" is a
  scoped session allowlist (tool + arg/path prefix) that kills approval fatigue. Reject folds
  into a steering redirect. Codex adds coarse session modes (Read-Only / Auto / Full-Access);
  Claude adds `Shift+Tab` to cycle normal / auto-accept-edits / plan, with the active mode
  always visible near the composer.
- **Progressive disclosure:** tool calls render as `● Tool(arg)` with the result bound under a
  `⎿` connector; long output collapses to `… +N lines (ctrl+r to expand)`; prose full-bright,
  machinery dimmed.
- **Interrupt is cheap and non-destructive:** `Esc` / first `Ctrl+C` aborts the turn (and kills
  the child process) and returns to the composer; second `Ctrl+C` quits. Cheap bail is what makes
  users grant autonomy.
- **Input:** `Enter` submits, `Shift+Enter` newline (with a backslash-Enter universal fallback
  because terminals disagree on Shift+Enter bytes), `/` opens a fuzzy command popup, history on
  Up/Down with the draft preserved in a virtual bottom slot.
- **Inline rendering (done):** the product shell now uses the main screen buffer with
  `Viewport::Inline` sticky chrome + `insert_before` for finalized transcript rows
  (`terminal.rs` / `terminal_app.rs`). Native scrollback/selection/copy are preserved;
  mouse capture is off. Click-to-expand is keyboard-only (`Ctrl+P` / `Ctrl+O`) under this
  model.

---

## 4. Target design

### 4.1 Layout (vertical stack, bottom-anchored)

```
┌─────────────────────────────────────────────┐
│ transcript (Min, scrolls, sticky-to-bottom)  │  ← prose, tool calls (● / ⎿), inline diffs
│                                               │
├─────────────────────────────────────────────┤
│ ⠹ Working… 3.4s · ↑1.2k tok · esc interrupt   │  ← Length(1), ONLY while a turn is active
├─────────────────────────────────────────────┤
│ ❯ type a message — / for commands             │  ← composer (dynamic height, tui-textarea)
├─────────────────────────────────────────────┤
│ openai/gpt-5.5 · streaming · auto-edit · ctx 12k/128k · ~/agent │  ← persistent status footer
└─────────────────────────────────────────────┘
```

- The activity line is **its own row** between transcript and composer, not a border title.
  When idle it collapses (height 0) and the transcript reclaims the space.
- Approvals and destructive-edit diffs become **centered modal overlays** (`Clear` + bordered
  block — the slash-suggestion `render.rs:531` recipe already proves the primitive). Trace /
  Tasks / Sessions / Status / Extensions stay docked side panels.

### 4.2 A single source of turn truth

Introduce an explicit `TurnState` owned by `TuiApp`:

```
enum TurnState {
  Idle,
  Waiting,                 // turn started, no output yet (network/model warmup)
  Thinking,                // reasoning streaming
  Responding,              // final answer streaming
  ToolRunning { name },    // a tool is executing
  Cancelling,              // cooperative cancel requested
}
```

Fold `turn_active` + `active_cancellation` + `queued_prompts` into one `TurnController` owned by
the loop. Set `turn_started_at: Instant` at turn start (any path — fresh, queued, recovery),
clear on `Done`. Drive the spinner, the status/activity line, and redraw-gating from this — not
from scanning transcript streaming flags. This one change unlocks the correct spinner for queued/
recovery turns, an accurate elapsed timer, the reachable `Waiting` state, and cheap redraw gating,
and collapses the four `apply_terminal_effect*` overloads into one owner.

### 4.3 Event loop: dirty-flag + time-driven animation

- Block on input when idle (`event::poll(None)`); use a short timeout (~80–120ms) **only** when
  `TurnState != Idle` or an overlay animation is live.
- `terminal.draw()` only when `dirty` (a key/paste/resize consumed, a frame drained, or an
  animation tick advanced). Idle CPU → ~0%.
- Spinner frame = `(elapsed.as_millis() / 80) % FRAMES.len()` from `turn_started_at` — decoupled
  from the redraw clock, so it never freezes while typing and never changes speed when you tune
  the poll cadence. Keep the existing 10-frame braille array (`render.rs:456`).

### 4.4 Approvals (the safety spine)

- **Modal overlay** that captures all keys until resolved. An `overlay: Option<Overlay>` on
  `TuiApp`; when `Some`, keys route to the overlay handler first, skipping the composer keymap —
  this deletes the `composer.text.is_empty()` guards entirely.
- Show the **concrete action** in the modal: reuse the real diff renderer for file edits and the
  real command block for shell. Never a paraphrase.
- Three explicit choices, **safe default highlighted**, chosen by an unambiguous key (not bare
  Enter): `1 Allow once` / `2 Allow always (this tool + this path/arg, this session)` /
  `3 Reject & redirect` (opens a one-line reason that becomes the next steering message).
- `Esc` = non-destructive dismiss/defer (leaves the request pending), never deny.
- Persistent **status-footer indicator** whenever a request is pending, regardless of active
  panel or terminal width: `⚠ 1 approval pending`. Never armed-but-invisible.
- Add a scoped-rule store + an `AllowAlways` decision variant so repeat safe ops auto-approve.
- Render the pending queue as a selectable list when >1.
- Layer session modes on top: `Shift+Tab` cycles `normal / auto-accept-edits / plan`, with the
  active mode always shown in the footer.

### 4.5 Information taxonomy (kill the noise)

Five destinations, each with a distinct visual language:

1. **Transcript** — user / model / reasoning / tool (`●`/`⎿`) / approval-record / **real errors
   only**.
2. **Persistent status footer** — provider/model, connection/turn-state, permission mode, context
   tokens, workspace. One `ConnectionState` enum with a stable `Display`; both footer and Status
   panel read from it. Never surface `{:?}` Debug or raw status codes.
3. **Live activity line** — spinner + verb + elapsed + tokens + interrupt hint (while working).
4. **Transient toast / dim note** — no-op guidance and success confirmations (clipboard copy,
   "nothing to cancel"). Neutral/dim, never a red chip; auto-expires.
5. **Status side panel (drill-down)** — severity-colored diagnostics (error red / warn yellow /
   info dim), provider health, model attempts; scrollable. Not the only home for core state.

Give `LocalDiagnostic` a severity field (`Note | Warning | Error`). Delete the ~460 dead
`Progress` arms in `transcript.rs`.

### 4.6 Composer (adopt tui-textarea)

- Swap `ComposerState.text: String` for `tui_textarea::TextArea`. Keep the history ring and
  slash-suggestion layer on top (read `textarea.lines()`). Intercept `Enter`=submit /
  `Shift+Enter`=newline before delegating the rest to `TextArea::input`.
- Enable crossterm **bracketed paste** in `enter_selectable` (disable on restore) so multi-line
  paste arrives as one atomic `Event::Paste`. Add a regression test: a pasted string containing
  `\n` must not submit.
- Dynamic composer height (grow to N rows, capped) with wrap; cursor (x,y) from the real caret
  index across wrapped lines.
- Preserve the in-progress draft in a virtual slot when navigating history.
- On Enter with a slash-menu selection, complete-and-run in one step; reserve Tab for
  complete-without-running.
- Dim placeholder text in the empty composer: `type a message — / for commands · Enter to send`.

### 4.7 Keymap (convention-driven, one dispatcher)

| Key | Idle | Turn active | Overlay (approval) |
|-----|------|-------------|--------------------|
| `Enter` | submit | queue prompt | — (never bare-approve) |
| `Shift+Enter` / `\`+`Enter` | newline | newline | — |
| `Esc` | close panel / back | **interrupt turn** | dismiss (non-destructive) |
| `Ctrl+C` | quit | interrupt; 2× = force-quit | — |
| `Shift+Tab` | cycle permission mode | cycle mode | — |
| `/` | command popup | command popup | — |
| `Up`/`Down` | history (caret in multiline) | history | select queue item |
| `Ctrl+R` | expand last output | — | — |
| `1`/`2`/`3` | — | — | Allow-once / Allow-always / Reject |

Derive the footer hint text from the same dispatcher/state so hints can never drift from behavior.
Decide `Esc`-interrupt vs `Ctrl+C`-interrupt deliberately (see §5) and make the live-line hint
match whatever you choose.

### 4.8 Testing

- Keep `render_app_to_string` + `TestBackend`. Add `insta` snapshot tests for whole-frame layouts
  at fixed sizes (40×10 narrow, 88×24 standard, 120×40 wide, plus a wide-CJK case).
- Inject a fixed clock / elapsed into the render path so the time-driven spinner is deterministic.
- Snapshot the composited buffer for overlays to prove `Clear` removed transcript bleed-through.
- Add regression tests: multi-line paste doesn't submit; approval requires an explicit key (bare
  Enter does not approve); spinner advances while typing.

---

## 5. Decisions to make before coding

1. **Alternate screen vs inline rendering.** **Resolved: inline.** The product shell uses
   raw mode + bracketed paste on the main buffer, `Viewport::Inline` for sticky
   composer/status chrome, and `insert_before` to push finalized transcript lines into
   host scrollback. Alt-screen / mouse-capture entry points remain only as legacy aliases
   for tests.
2. **Interrupt key.** Claude Code/Codex use `Esc`; the repo uses `Ctrl+C` and prints
   "(Ctrl+C to interrupt)". A single always-safe `Esc` that never changes meaning is the
   competitive norm. Recommendation: make `Esc` the primary interrupt, keep `Ctrl+C` as
   interrupt+force-quit, and repoint the current bare-`Esc`=OpenChat behavior.
3. **Scoped allow-always store.** Where do session allow-rules live and what's the scope key
   (tool + argv[0] / path glob)? Needs a small design before implementing Cluster A fully.

---

## 6. Suggested phasing

1. **Phase 1 — Turn-state + event loop (Cluster E).** Explicit `TurnState`/`TurnController`,
   dirty-flag draw gating, time-driven spinner, stop rebuilding panels every frame. Unblocks
   everything else. No user-visible regressions; big CPU + correctness win.
2. **Phase 2 — Live activity line + status footer (Cluster B + C.2/C.3).** Move the indicator to
   its own row; make the footer carry connection/mode/permission/context; single `ConnectionState`.
3. **Phase 3 — Approvals modal (Cluster A).** Overlay + focus stack, safe-default keys, persistent
   pending indicator, reason capture, allow-always scoped rules, `Shift+Tab` modes.
4. **Phase 4 — Composer (Cluster D).** Adopt tui-textarea, enable bracketed paste, dynamic height,
   history draft preservation, placeholder.
5. **Phase 5 — Noise taxonomy + polish (Cluster C).** Severity on `LocalDiagnostic`, toasts,
   delete dead `Progress` arms, Status panel as styled drill-down, inline diffs with `⎿` +
   collapse, help/welcome copy fixes.
6. **Phase 6 — Snapshot test migration + accessibility.** `insta` golden frames, injected clock,
   centralized `Palette`, color-plus-glyph for every semantic state.

Each phase ends green (`cargo test -p ruse-tui`, `cargo clippy`) and adds/updates tests before
moving on.
