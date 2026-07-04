# M29-03 GitHub Actions Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a repository CI workflow that delegates to `cargo xtask ci`.

**Architecture:** GitHub Actions is only an invocation layer; quality logic stays in `xtask`.

**Tech Stack:** GitHub Actions YAML, xtask static tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M29-03 spec and current xtask CI command**

### Task 2: Write Tests

- [x] **Step 1: Add static workflow validation test**

### Task 3: Implement

- [x] **Step 1: Add `.github/workflows/ci.yml`**

### Task 4: Verify and Commit

- [x] **Step 1: Run focused workflow checks**

```bash
cargo fmt --all -- --check
cargo test -p xtask github_actions_workflow
```

```bash
git add .github xtask docs
git commit -m "ci: add xtask quality workflow"
```
