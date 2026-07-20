# M13-03 Invocation Policy Spec

## Purpose

Add timeout, retry, and fallback policy contracts for model invocation.

## Product Context

Real provider calls can fail due to timeout, rate limit, transient transport failure, invalid credentials, or unsupported requests. Runtime behavior must be explicit, traceable, and safe to replay.

## Scope

This spec delivers:

- `ModelInvocationPolicy`
- `ModelRetryPolicy`
- `ModelFallbackChain`
- retry classification
- fallback eligibility checks

Owned by later specs:

- live provider transport
- scheduler-level model quota accounting

## Architecture

Invocation policy lives in `ruse-model` as provider-domain decision data. `ruse-core` may later wrap a selected provider with policy execution and event emission.

## Acceptance Criteria

- timeout and retry settings validate
- retry classifier distinguishes retryable and terminal failures
- fallback chain rejects duplicate or empty provider ids
- fallback requires compatible requested capabilities
- decisions serialize into stable trace-safe records
