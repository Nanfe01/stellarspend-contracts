# `<contract-name>`

> _One-line description of what this contract does._

## Overview

<!--
  2–3 sentences expanding on the one-liner.
  Answer: WHY does this contract exist and WHAT problem does it solve?
-->

## Features

<!--
  Bullet list of the contract's key capabilities.
  Focus on user-facing behaviour, not internal implementation details.
-->

- **Feature A**: Brief description
- **Feature B**: Brief description

---

## Public API

<!--
  List every `pub fn` inside the `#[contractimpl]` block.
  Group by lifecycle (init → write → read) for readability.
-->

### Initialization

```rust
pub fn initialize(env: Env, admin: Address, /* ... */)
```

_Short description of parameters and behaviour._

### Write Functions

```rust
pub fn some_write_fn(env: Env, caller: Address, /* ... */) -> ReturnType
```

| Parameter | Type      | Description             |
| --------- | --------- | ----------------------- |
| `caller`  | `Address` | Must be authorized      |
| `...`     | `...`     | ...                     |

_Describe the function's behaviour, any side-effects, and emitted events._

### Read / Query Functions

```rust
pub fn some_read_fn(env: Env, /* ... */) -> ReturnType
```

_Short description._

---

## Storage Layout

<!--
  Document the `DataKey` enum and explain how each variant maps to
  Soroban storage (instance vs persistent vs temporary).
  This is crucial for contributors to reason about rent, TTLs, and upgrades.
-->

| DataKey Variant       | Storage Type | Value Type | Description                     |
| --------------------- | ------------ | ---------- | ------------------------------- |
| `Admin`               | Instance     | `Address`  | Contract administrator          |
| `SomeRecord(u64)`     | Persistent   | `Record`   | Stored record by ID             |

---

## Events

<!--
  List all events emitted by this contract.
  Include topic structure and payload for indexer integration.
-->

| Event Name         | Topics                              | Payload                           |
| ------------------ | ----------------------------------- | --------------------------------- |
| `thing_created`    | `("thing", "created")`              | `(id, user, amount)`              |

---

## Error Codes

<!--
  If the contract defines custom error enums, document every variant.
-->

| Code | Name              | Description                    |
| ---- | ----------------- | ------------------------------ |
| 1    | `NotInitialized`  | Contract has not been set up   |
| 2    | `Unauthorized`    | Caller lacks required role     |

---

## Types

<!--
  Document key structs and enums used in the public API.
  Omit internal-only types.
-->

### `SomeStruct`

```rust
pub struct SomeStruct {
    pub field_a: Type,
    pub field_b: Type,
}
```

---

## Usage Example

<!--
  Short, self-contained example showing how to call the contract.
  Prefer test-style pseudocode using Soroban testutils.
-->

```rust
// 1. Initialize
contract.initialize(&env, &admin);

// 2. Call a write function
let result = contract.some_write_fn(&env, &caller, arg);

// 3. Read state
let value = contract.some_read_fn(&env, id);
```

---

## Testing

```bash
cargo test -p <contract-name>
```

---

## Design Notes

<!--
  Optional section for non-obvious architectural decisions,
  optimisation strategies, or known limitations.
  Delete this section if not applicable.
-->

---

## License

MIT
