# AGENTS.md — Looker

## Governance
- Constitution: [`constitution/CONSTITUTION.md`](constitution/CONSTITUTION.md)
- Operating Model: [`organization/profiles/release-driven-solo.md`](organization/profiles/release-driven-solo.md)

## Safety / evidence
- Treat filesystem mutation as a consequential side effect.
- Prefer dry-run/plan inspection before `--apply`; dry-run is not proof of applied results.
- Tests and agent validation must use disposable fixture roots, never unrelated user record/project directories.
- Cargo manifests/lockfiles are Rust dependency authority; do not create a second Rust toolchain pin in mise.
- Release evidence binds binary, target platform, source SHA, and validation result.

## Delivery
- durable work: GitHub Issue.
- ticket branch: Issue number.
- ticket PR: current release branch.
- main integration: release PR only.
- landing: merge commit only.
