# Event Schema

## Overview

This document defines the standard event payloads emitted by the StellarSpend contracts.

## Common Fields

Every event payload should include the following fields where applicable:

| Field | Type | Description |
|------|------|-------------|
| event_type | Symbol | Type of event |
| amount | i128 | Amount involved in the event |
| timestamp | u64 | Ledger timestamp |
| contract | Address | Contract emitting the event |

## Treasury Events

### Penalty Received

Topics

```
("treasury", "penalty")
```

Payload

```rust
{
    amount: i128,
    timestamp: u64,
}
```

### Fee Received

Topics

```
("treasury", "fee")
```

Payload

```rust
{
    amount: i128,
    timestamp: u64,
}
```

### Reward Received

Topics

```
("treasury", "reward")
```

Payload

```rust
{
    amount: i128,
    timestamp: u64,
}
```