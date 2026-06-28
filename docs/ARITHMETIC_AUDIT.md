# Arithmetic Overflow/Underflow Audit

## Summary
Audit of workspace arithmetic on balance/amount types to confirm
checked or saturating operations are used.

## Findings

### Safe (checked arithmetic confirmed)
- `contracts/fee/src/safe_sub.rs` — dedicated safe subtraction utility
- `contracts/streak_reward.rs` — uses `checked_add`, panics with `StreakError::Overflow`
- `contracts/batch-history` — uses checked arithmetic throughout

### Requires Attention
- `contracts/metadata.rs` — uses `serde`-based approach, no direct arithmetic
- `contracts/state.rs` — cosmwasm Uint128 is overflow-safe by design

## Conclusion
Core balance-affecting contracts use checked arithmetic.
No raw `+`/`-` on balance types found in high-risk paths.
Remaining findings have been ticketed for follow-up.
