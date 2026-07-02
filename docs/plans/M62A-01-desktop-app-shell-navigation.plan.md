# M62A-01 Desktop App Shell and Navigation Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly.

**Goal:** Replace the current route-centric Desktop layout with a dense three-column coding workspace shell.

**Architecture:** React owns shell presentation state. Existing Desktop snapshots and bridge frames remain runtime truth. Tauri and Rust runtime logic are not changed by this slice unless a typed snapshot is missing and explicitly planned.

**Tech Stack:** React 18, TypeScript, Vite, Vitest, CSS variables/plain CSS initially, optional Radix UI primitives/CVA/lucide-react after build-impact review.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M62A-01-desktop-app-shell-navigation.spec.md`
- Read: `apps/desktop/package.json`
- Read: `apps/desktop/src/App.tsx`
- Read: `apps/desktop/src/app.css`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/__tests__/app-shell.test.tsx`

- [ ] **Step 1: Review current shell**

Run:

```bash
sed -n '1,620p' apps/desktop/src/App.tsx
sed -n '1,620p' apps/desktop/src/app.css
```

Expected: identify current dashboard/conversation/tasks/approvals/trace/settings route layout and reusable components.

### Task 2: Evaluate UI Foundation

**Files:**
- Modify: `apps/desktop/package.json` only if dependencies are adopted
- Modify: `docs/plans/M62A-01-desktop-app-shell-navigation.plan.md` with dependency notes if needed

- [ ] **Step 1: Decide dependency footprint**

Evaluate whether to add `@radix-ui/*`, `class-variance-authority`, and `lucide-react`. If added, keep the set minimal and run install/build checks.

### Task 3: Implement Shell State and Layout

**Files:**
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/app.css`
- Modify/Create: `apps/desktop/src/components/*`
- Modify/Create: `apps/desktop/src/state/*`

- [ ] **Step 1: Add shell UI state**

Add typed shell state for sidebar, inspector, terminal drawer, command palette, and active inspector tab.

- [ ] **Step 2: Build layout regions**

Render left sidebar, center transcript/composer, right inspector tabs, and bottom terminal drawer placeholder with responsive constraints.

- [ ] **Step 3: Add command palette and shortcuts**

Add Cmd/Ctrl+K command palette skeleton and shortcut handlers for new thread, toggles, settings, and find-in-thread.

### Task 4: Verify

**Files:**
- Modify: `apps/desktop/src/__tests__/app-shell.test.tsx`

- [ ] **Step 1: Add focused tests**

Cover shell regions, sidebar sections, inspector tabs, terminal drawer, command palette, and keyboard shortcuts.

- [ ] **Step 2: Run checks**

Run:

```bash
npm --prefix apps/desktop test -- app-shell
npm --prefix apps/desktop run build
```

Expected: focused tests and production build pass.

### Task 5: Commit

**Files:**
- Stage only M62A-01 source/tests/docs.

- [ ] **Step 1: Commit slice**

Run:

```bash
git diff --check
git add apps/desktop docs/specs/M62A-01-desktop-app-shell-navigation.spec.md docs/plans/M62A-01-desktop-app-shell-navigation.plan.md
git commit -m "feat: productize desktop app shell"
```

Expected: commit succeeds without staging unrelated recovery files.
