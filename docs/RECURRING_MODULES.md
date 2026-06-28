# Recurring Module Boundaries

## Overview
Three recurring-related modules exist in this workspace:

## `contracts/recurring/`
**Owner:** General recurring execution engine.
Contains `executor.rs`, `scheduler.rs`, `types.rs`, `mod.rs`.
Responsible for: schedule creation, execution, pause/resume logic.

## `contracts/recurring-payment/`
**Owner:** Payment-specific recurring logic.
Contains `lib.rs`, `test.rs`, `types.rs`.
Responsible for: recurring payment execution using the token interface.
Depends on `recurring/` for scheduling primitives.

## `contracts/recurring_savings.rs`
**Owner:** Savings-specific recurring contributions.
A single-file module for recurring savings deposits.
Distinct from payments: no token transfer, only savings goal tracking.

## Decision
No functional duplication found. Each module has a clear boundary.
Naming is intentionally distinct: `recurring/` (engine), `recurring-payment/` (payments), `recurring_savings.rs` (savings).
