# Bootstrap the Rewards Smart Contract

**Closes #875**

---

## Summary

Introduces the `rewards` crate — a new, isolated Soroban smart contract that lays the foundation for reward management within the StellarSpend protocol. The contract is intentionally minimal per the issue scope, providing a clean, tested base that future reward functionality can build on top of.

---

## Changes

### New: `contracts/rewards/`

- **`Cargo.toml`** — workspace-inherited manifest (`version`, `edition`, `license`, `repository`); configured as `cdylib` with a `testutils` feature flag, matching the conventions of other contracts in the workspace.
- **`src/lib.rs`** — `RewardsContract` with three public entry points:
  - `initialize(env, admin)` — one-shot initialisation guarded against double-calls; emits an `initialized` event.
  - `get_admin(env)` — returns the current admin address; panics with `NotInitialized` if called before init.
  - `is_initialized(env)` — lightweight boolean state check.
  - Internal `require_admin` helper scaffolded for future reward operations.
- **`src/test.rs`** — 4 unit tests covering all acceptance criteria:
  - `test_initialize_sets_admin` — init stores and returns the correct admin.
  - `test_is_initialized_returns_true_after_init` — flag is `false` before init, `true` after.
  - `test_double_initialize_panics` — second call is rejected.
  - `test_get_admin_before_init_panics` — getter panics cleanly when uninitialised.

### Modified: `Cargo.toml` (root)

- Registered `contracts/rewards` in the workspace `[members]` list.
- Fixed a pre-existing malformed `[[test]]` block (stray non-TOML text + duplicate `name`/`path` keys) that was preventing `cargo fetch` and `cargo test --workspace` from running.

---

## Test Results

```
running 4 tests
test test::test_get_admin_before_init_panics - should panic ... ok
test test::test_double_initialize_panics - should panic ... ok
test test::test_initialize_sets_admin ... ok
test test::test_is_initialized_returns_true_after_init ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

---

## Out of Scope

This PR does not implement any reward distribution logic — that is intentionally deferred to follow-on issues. The contract is isolated from existing budgeting and savings contracts as required.
