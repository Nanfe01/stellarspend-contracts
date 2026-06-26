# `zk-verifier`

> Verify UltraHonk zero-knowledge proofs on-chain to authorize privacy-preserving spending.

## Overview

The `zk-verifier` contract is the on-chain proof verification component of StellarSpend's privacy-preserving spending limits system. It receives an UltraHonk proof generated off-chain by a Noir circuit and verifies that a payment is within the user's spending limit — **without revealing the payment amount on-chain**.

This contract is the gateway to the `spending-limits` contract: no payment can be authorized unless the ZK proof passes verification here first.

## Features

- **On-Chain ZK Verification**: Verifies UltraHonk proofs submitted from off-chain Noir circuits
- **Privacy-Preserving**: Payment amounts never appear on-chain
- **Lightweight Interface**: Single verification function with a boolean result
- **Composable**: Designed to be called by the `spending-limits` contract before authorizing payments

---

## Public API

### `verify_spending_proof`

```rust
pub fn verify_spending_proof(
    _env: Env,
    _user: Address,
    proof: Bytes,
) -> bool
```

| Parameter | Type      | Description                                                     |
| --------- | --------- | --------------------------------------------------------------- |
| `_user`   | `Address` | The user whose spending is being verified (reserved for future) |
| `proof`   | `Bytes`   | The serialized UltraHonk proof bytes                            |

**Behaviour:**

1. Checks that `proof` is non-empty. Returns `false` if the proof has zero length.
2. Returns `true` if the proof passes validation.

> **Note:** The current implementation is a proof-of-concept that validates proof presence. Full UltraHonk cryptographic verification is planned once Soroban host functions support the required elliptic curve operations.

**Returns:** `bool` — `true` if the proof is valid, `false` otherwise.

---

## Storage Layout

This contract is **stateless** — it does not read from or write to any storage. It operates as a pure verification function.

---

## Events

No events are emitted by this contract. The calling contract (`spending-limits`) is responsible for emitting authorization events.

---

## Types

No custom types are defined. The contract uses only SDK primitives (`Bytes`, `Address`, `bool`).

---

## Usage Example

```rust
use soroban_sdk::{Env, Address, Bytes};
use zk_verifier::ZkVerifierContract;

// Generate proof off-chain using Noir + Barretenberg
let proof: Bytes = /* serialized UltraHonk proof */;

// Verify on-chain
let is_valid = contract.verify_spending_proof(&env, &user, &proof);

if is_valid {
    // Proceed to spending-limits contract for authorization
}
```

### End-to-End Flow

```text
1. User generates proof:  nargo execute → bb prove
2. Proof submitted to:    zk-verifier::verify_spending_proof()
3. If valid:              spending-limits contract authorizes payment
4. If invalid:            Transaction rejected — amount stays private
```

---

## Testing

```bash
cargo test -p zk-verifier
```

---

## Design Notes

- **Stateless by design**: The verifier has no storage, no initialization, and no admin. This makes it simple to upgrade or replace without migration concerns.
- **Proof-of-concept verification**: The current `proof.len() > 0` check is a placeholder. Full Barretenberg/UltraHonk verification requires host-level support for BN254 pairing operations, which are not yet available in Soroban.
- **Separation of concerns**: Verification logic is isolated from spending authorization. The `spending-limits` contract calls this contract and decides what to do based on the boolean result.

---

## Related

- [ZK Circuit](../../circuits/spending_proof/src/main.nr) — The Noir circuit that generates the proof
- [Proof Generation Script](../../scripts/generate_proof.sh) — Shell script for local proof generation

---

## License

MIT
