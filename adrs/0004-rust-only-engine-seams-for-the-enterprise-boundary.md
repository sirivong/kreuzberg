---
status: accepted
date: 2026-06-29
decision-makers: ["Na'aman Hirschfeld"]
---

# Rust-only engine seams for the enterprise boundary

## Context and Problem Statement

xberg-enterprise's worker consumed `xberg` through the high-level `extract()` and reimplemented machinery around it:
a near-copy of `xberg::heuristics`, its own model loading, and repeated parsing of one document. Its ADR-0049
(2026-06-27) set the requirement for this repository: the MIT core holds the mechanism, and the commercial value
(curated presets, managed-LLM policy, tenant policy, distributed infrastructure) stays in enterprise as trait
implementations injected through public seams with working in-core defaults. The license split itself is
[org ADR-0003 (Licensing line)](https://github.com/xberg-io/agent-conventions/blob/main/adrs/0003-licensing-line.md).

## Considered Options

* Rust-only extension seams under `xberg::engine::seams`, each with an in-core default, kept off the language-binding
  surface
* Add ad hoc knobs to `extract()` only
* Move the worker's structured-extraction orchestrator into the core

The two alternatives are the ones enterprise ADR-0049 recorded and rejected.

## Decision Outcome

Chosen option: "Rust-only extension seams under `xberg::engine::seams`, each with an in-core default, kept off the
language-binding surface", because enterprise ADR-0049 found that knobs on `extract()` give no injection point for
commercial value, and moving the orchestrator would pull prompt packs, citation presentation and managed-key policy
into the MIT core.

475f83ad39 (2026-06-29) added six seams (`CacheBackend`, `ProgressSink`, `StructuredPolicy`, `PresetResolver`,
`LlmClient`, `ModelProvider`), each with a behavior-preserving default, and kept them out of the bindings: their
files are not in `alef.toml` sources, nothing is re-exported from `lib.rs`, and every seam type is in
`[crates.exclude]`. ddba546dca (2026-08-25) narrowed injection to the two seams the extraction path reads:
`EngineBuilder` takes only a `CacheBackend` and a `ProgressSink`, and the other four are standalone primitives that a
caller composes where it runs that operation. Enterprise records that narrowing as its ADR-0076.

### Consequences

* Good, because enterprise injects caching and progress into the same engine every binding uses, without a fork of
  the core.
* Good, because the seams stay a Rust-only API, so the binding surface and its compatibility promises do not grow
  with them.
* Bad, because enterprise depends on Rust primitives that no binding exposes, so removing or reshaping one is a
  breaking change for enterprise even when no binding changes. The polyrepo-boundaries skill says to check
  `xberg-enterprise` first.

### Confirmation

* `crates/xberg/src/engine/seams/mod.rs:3` states that cache and progress are injectable and the remaining traits
  are not engine configuration.
* `crates/xberg/src/engine/mod.rs:252` and `:258` are the only `EngineBuilder` setters, `with_cache_backend` and
  `with_progress_sink`.
* `alef.toml` lists the seam types under `[crates.exclude]` (`:3061`), for example
  `xberg::engine::seams::CacheBackend` at `:3105`.
* The polyrepo-boundaries skill (`.ai-rulez/skills/polyrepo-boundaries/SKILL.md`) assigns product and deployment
  behavior to `xberg-enterprise`.
