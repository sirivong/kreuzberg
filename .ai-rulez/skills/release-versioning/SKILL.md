---
name: release-versioning
description: How xberg versions are synced and released — Cargo.toml is the single source of truth, `task version:sync` propagates it to alef-managed binding manifests AND the integrations under integrations/, which are versioned and published in lockstep with core (including -rc.N). Load before bumping a version, editing the version-sync task, or touching an integration's version/xberg dependency.
---

# Release & Versioning

## Single source of truth

The root `Cargo.toml` `version` is the one authoritative version (including any
`-rc.N` pre-release suffix). Everything else is derived from it — never hand-edit
a version in a package manifest.

## `task version:sync` propagates to two families

`task version:sync` (alias `task versions:sync`) runs, in order:

1. `alef sync-versions` — updates the **alef-managed binding manifests** (their own
   version = core version). Targets are listed in `alef.toml` `[workspace.sync] extra_paths`
   (packages/python, packages/ruby, crates/xberg-node, packages/go, cli-proxy, …).
2. `python3 scripts/sync_integration_versions.py` — updates the **integrations**
   under `integrations/`. These are NOT alef-managed, so alef never touches them.

Bump/set helpers chain both automatically:
`task version:bump:major|minor|patch`, `task version:set -- <version>`.
`task version:check` dry-runs both and fails on drift (`sync_integration_versions.py --check`).

## Integrations are lockstep with core

The integration packages under `integrations/` (langchain, llama-index readers +
node-parser, crewai, txtai, surrealdb — Python; spring-ai — Java) are versioned and
**published together with core**. `scripts/sync_integration_versions.py` sets, for each:

- the package's own `version` — PEP 440 form for pyproject (`1.0.0-rc.32` → `1.0.0rc32`),
  native form for the Maven pom (`1.0.0-rc.32`);
- the `xberg` dependency floor (`xberg>=<core>` / `<xberg.version>`), so an integration
  always requires the core it ships with. Naming the rc in the specifier is deliberate —
  a bare `xberg>=1.0.0` excludes all `1.0.0rcN` pre-releases per PEP 440.

To add a new integration: add its manifest to `VERSION_TARGETS` (own version) and, if it
depends on xberg, `XBERG_DEP_MANIFESTS` in `scripts/sync_integration_versions.py`. The
llama-index dev aggregator (`integrations/python/llama-index/pyproject.toml`, version
`0.0.0`, unpublished) is dep-only — not a version target.

## Do

- Bump via `task version:bump:*` / `task version:set`, then commit the synced manifests
  together with the Cargo.toml change (atomic).
- Run `task version:check` in CI to guarantee integration manifests never drift from core.

## Don't

- Don't hand-edit a package/manifest version or an integration's `xberg` pin — run the sync.
- Don't add integration manifests to `alef.toml` `[workspace.sync]` — alef would clobber
  their independent-but-derived layout; the dedicated script owns them.
