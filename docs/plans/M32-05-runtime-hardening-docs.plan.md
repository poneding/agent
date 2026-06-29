# M32-05 Runtime Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M32 documentation, hardening checks, and final milestone status.

**Architecture:** Documentation and final verification slice.

**Tech Stack:** Markdown docs, Cargo workspace quality gates.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M32 specs and completed implementation slices**

### Task 2: Update Docs

- [ ] **Step 1: Update README profile manifest usage**
- [ ] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [ ] **Step 1: Run final M32 focused tests**
- [ ] **Step 2: Run `cargo xtask ci`**

### Task 4: Commit

```bash
git add README.md docs crates
git commit -m "feat: complete m32 mcp export profile runtime"
```
