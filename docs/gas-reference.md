# Gas-Cost Reference

A consolidated reference for the expected gas (Soroban budget) costs of the
StellarSpend contracts. It aggregates the benchmark data that already lives in
the codebase so contributors have a single place to look up expected costs and
the regression ceilings the test suite enforces.

Soroban's `Budget` tracks two resources:

- **CPU instructions** — the computational effort of a call. Lower is better.
- **Memory bytes** — the host memory allocated during a call. Lower is better.

These are deterministic for a given contract version, so the numbers below are
reproducible from the benchmark code rather than wall-clock timings.

> **Sources aggregated by this document**
> - [`contracts/benchmarks/src/lib.rs`](../contracts/benchmarks/src/lib.rs) — measured budget benchmarks
> - [`contracts/gas-optimization.rs`](../contracts/gas-optimization.rs) — staking / batch / escrow budget ceilings
> - [`contracts/batch-rewards/src/gas.rs`](../contracts/batch-rewards/src/gas.rs) — batch-reward storage-op accounting
> - [`contracts/escrow/src/gas.rs`](../contracts/escrow/src/gas.rs) — escrow optimization notes
> - [`contracts/events/src/gas.rs`](../contracts/events/src/gas.rs) — event-emission optimization notes

---

## 1. Measured budget benchmarks

These figures are the **actual** CPU and memory budget consumed by a single
operation, captured by running the benchmark suite:

```bash
cargo test --package stellarspend-benchmarks --release -- --nocapture
```

| Operation                          | Contract        | CPU instructions | Memory (bytes) | Regression ceiling (CPU) |
| ---------------------------------- | --------------- | ---------------: | -------------: | -----------------------: |
| Budget creation (initial record)   | `budget`        |          126,502 |         19,892 |              < 5,000,000 |
| Spending validation (`spend_from_category`) | `budget` |          159,965 |         26,269 |              < 5,000,000 |
| Goal creation (`batch_set_savings_goals`, n=1) | `savings-goals` |     251,143 |         32,206 |              < 8,000,000 |
| Goal contribution (`contribute_to_goal`) | `savings-goals` |       202,169 |         34,912 |              < 5,000,000 |
| Goal withdrawal (`withdraw_from_goal`)   | `savings-goals` |        89,303 |         16,942 |              < 5,000,000 |
| Event-emission overhead (approx. via `update_budget`) | `budget` | 137,789 |   21,154 |              < 2,000,000 |

Measured values are well under their ceilings — the ceilings exist to catch
regressions (for example, someone reintroducing redundant storage reads), not to
pin exact numbers.

---

## 2. Staking, batch-reward, and escrow budget ceilings

The benchmarks in [`contracts/gas-optimization.rs`](../contracts/gas-optimization.rs)
assert upper bounds on the CPU budget for the staking, batch-reward, and escrow
flows. These ceilings are the documented expected-cost envelope for each
operation.

### Staking

| Operation    | Behaviour                                | Regression ceiling (CPU) |
| ------------ | ---------------------------------------- | -----------------------: |
| `stake`      | single packed `StakeEntry` read + write  |              < 5,000,000 |
| `unstake`    | single read + single write               |              < 5,000,000 |
| `get_stake`  | pure read, no reward computation         |              < 1,000,000 |
| `get_config` | instance-storage read (host-cached)      | 2nd read ≤ 1st + 100,000 |

### Batch rewards

| Recipients | Regression ceiling (CPU) |
| ---------: | -----------------------: |
|          1 |              < 3,000,000 |
|         10 |             < 15,000,000 |
|         50 |             < 60,000,000 |

Batch cost is expected to scale **sub-linearly** in the number of recipients:
the one-time setup (config read, auth check) is not repeated per user, so
`cost(50) / cost(1)` stays well below 50×.

### Escrow

| Operation        | Behaviour                              | Regression ceiling (CPU) |
| ---------------- | -------------------------------------- | -----------------------: |
| `lock`           | single packed `EscrowEntry` write      |              < 5,000,000 |
| `release`        | single read, removes slot (reclaims rent) |           < 5,000,000 |

---

## 3. Storage-operation accounting (batch rewards, 100-user batch)

From [`contracts/batch-rewards/src/gas.rs`](../contracts/batch-rewards/src/gas.rs),
the optimized batch-reward path versus a naïve implementation:

| Operation          | Naïve    | Optimized |
| ------------------ | -------: | --------: |
| Config reads       |      100 |         1 |
| `StakeEntry` reads |      100 |       100 |
| `StakeEntry` writes|      100 |     ≤ 100 |
| Events emitted     |      100 |         1 |
| **Total ops**      | **400+** |  **~202** |

`StakeEntry` reads are unavoidable (one per user); writes are skipped for users
with a zero balance; the config is read once before the loop; and a single
summary event replaces N per-user events, saving `(N - 1) * event_base_cost`.

---

## 4. Optimization notes by contract

The cost numbers above follow from these implementation choices.

### Escrow ([`escrow/src/gas.rs`](../contracts/escrow/src/gas.rs))

- One packed `EscrowEntry` per escrow ID instead of separate amount / unlock-ts keys.
- `release` removes the storage slot entirely, reclaiming ledger rent.
- Config is read once into a local — no repeated instance-storage lookups.
- A single `token::Client` is constructed per call, not once per branch.

### Batch rewards ([`batch-rewards/src/gas.rs`](../contracts/batch-rewards/src/gas.rs))

- Config read once before the loop, not once per recipient.
- Per-user computation done in memory; storage written at most once per user.
- One `BatchRewardEventData` summary instead of N individual events.
- Zero-balance users are skipped before any storage is touched.
- Storage slots are removed when a balance drops to zero (reclaims rent).

### Events ([`events/src/gas.rs`](../contracts/events/src/gas.rs))

- Topics are a fixed 2-tuple `(CONTRACT_TOPIC, op_topic)` — Soroban charges per
  topic element, so this is the minimum that still allows off-chain filtering.
- Payload structs carry only primitive / already-owned values, so emit helpers
  perform no extra heap allocation before publishing.
- `validate_*` guards are `#[inline]`, folded into the caller on the hot path.

---

## 5. Reproducing these figures

```bash
# Measured benchmark figures (Section 1)
cargo test --package stellarspend-benchmarks --release -- --nocapture

# Full test suite, including the budget-ceiling assertions (Sections 2–3)
cargo test --workspace
```

See [`contracts/benchmarks/README.md`](../contracts/benchmarks/README.md) for
more on running and interpreting the benchmark output.

When an optimization or contract change moves these numbers, update the figures
in this document so it stays in sync with the benchmark output.
