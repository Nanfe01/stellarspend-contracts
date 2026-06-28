# TODO - single PR: #770 security suite + #769/#779/#780 features

## 1) Security E2E (#770)
- [ ] Replace placeholder panics in `tests/security_e2e_tests.rs` with real contract interactions.
- [ ] Implement unauthorized withdrawals test using multisig savings withdrawal helpers.
- [ ] Add replay/idempotency tests via duplicate idempotency tokens.
- [ ] Add privilege escalation + storage manipulation tests against admin/owner auth gates.
- [ ] Add budget bypass tests (frozen/suspended budgets).

## 2) Automatic Budget Renewal (#769)
- [ ] Implement renewal frequency scheduling + executor calls.
- [ ] Clone budget state and persist historical budgets using budget history/version APIs.
- [ ] Add tests asserting renewal creates new budget version while preserving history.

## 3) Savings Goal Beneficiary Transfer (#779)
- [ ] Implement beneficiary/ownership reassignment path (budget-level preferred) with strict ownership checks.
- [ ] Emit audit event on ownership/beneficiary update.
- [ ] Add tests for secure transfer and rejection of unauthorized reassignment.

## 4) Multi-Goal Auto Allocation (#780)
- [ ] Implement multi-goal deposit splitting using allocation percentages (sum to 100 validation).
- [ ] Wire deposit/allocation path to create contributions across multiple goals.
- [ ] Add tests ensuring allocation totals match and replay protection works.

## 5) PR hygiene
- [ ] Create branch `blackboxai/<name>`.
- [ ] Ensure `security_e2e_tests` is wired in root `Cargo.toml`.
- [ ] Run `cargo test --test security_e2e_tests` and relevant unit tests in an environment with MSVC `link.exe`.

