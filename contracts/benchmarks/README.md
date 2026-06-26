# Benchmarks

## Running the benchmarks

```bash
cargo bench
# or, if the benchmarks are in a separate crate:
cargo run --package benchmarks --release
```

## Interpreting the output

The benchmark output includes columns for:

- **name** – the benchmark name.
- **gas** – estimated gas usage for the operation.
- **cpu_cycles** – CPU cycles consumed.
- **time** – wall‑clock execution time.

Example output:

```
benchmark_name            gas      cpu_cycles    time
----------------------------------------------------
budget_create            12345    67890         12.3ms
budget_withdraw          23456    78901         15.7ms
```

Higher **gas** values indicate higher cost on Soroban, while **cpu_cycles** reflect the computational effort required. Use these metrics to compare performance across contract changes.

## Screenshots

*(Add screenshots of benchmark runs here)*
