# Waseki R1CS DSL

This crate exposes a lightweight API for building Rank-1 Constraint Systems (R1CS) in pure Rust. The core pieces are intentionally simple:

1. `ConstraintSystem<F>` – wraps the thread-local `LocalState` and gates witness allocation/constraint emission.
2. `Var<F>` – represents a field element together with its linear combination.
3. `CompiledR1CS<F>` – holds the result (inputs, witnesses, and sparse A/B/C matrices).

## State lifecycle

`ConstraintSystem::new()` initialises a thread-local `LocalState`. When the `ConstraintSystem` is dropped, the state is removed automatically. All `Var` operations check whether a state is active. If you build a `Var` without running inside a `ConstraintSystem`, it acts as a pure value (no witness allocation, no constraints) and operations simply manipulate concrete field values.

```rust
use ark_bn254::Fr;
use waseki::{ConstraintSystem, Var};

let mut cs = ConstraintSystem::<Fr>::new();

// simple Fibonacci: f0=1, f1=1, enforce f2=f0+f1
let f0 = cs.input(Fr::one());
let f1 = cs.input(Fr::one());
let f2 = f0 + f1;
cs.inputize(f2);

// optionally enforce another step: f3 = f1 + f2
let f3 = f1 + f2;
cs.inputize(f3);

let compiled = cs.compile(); // stores inputs, witness, A/B/C matrices
```

Outside a `ConstraintSystem`, you can still use `Var::from`, `Var::one`, `+`, `-`, `*`, `Sum`, and `Product` to manipulate field values; they just won’t emit constraints.

## Modules

- `state.rs` – thread-local storage, allocation, serialization, and R1CS matrix expansion helpers.
- `list.rs` – fixed-size list that collects linear terms and spills into `LocalState`.
- `ops.rs` – arithmetic and aggregate trait implementations for `Var`.
- `var.rs` – user-facing API (`Var`, `ConstraintSystem`, `CompiledR1CS`).

Integration tests live under `tests/var.rs` and cover both stateful and stateless usage.
