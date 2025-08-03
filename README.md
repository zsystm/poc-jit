```sh
docker buildx build \
  --platform linux/amd64 \
  -t evm_poc_image \
  --load \
  .

docker run -it --rm evm_poc_image /bin/bash

cargo run --release
```

⚠️ **Performance Note for macOS Users**: Running this project on macOS with Docker may result in degraded performance that does not accurately reflect the JIT implementation's true capabilities. For reliable performance benchmarking, we recommend running on native Linux or using a Linux VM instead of Docker on macOS.

## Running experiments

Running `cargo run` generates random bytecode sequences exercising opcodes like
`PUSH`, `SLOAD`, `SSTORE`, `ADD`, `SUB` and compares an interpreter against the
JIT implementation. Results, including the executed bytecode and timing
information, are written to files under `reports/`.

## Example Benchmark Results

Here's an example of the performance improvements achieved by the JIT implementation:

```
BYTECODE JIT vs INTERPRETER BENCHMARK SUMMARY
==============================================

Test configurations:
  small: 10 cases, 20 opcodes
  medium: 10 cases, 100 opcodes
  large: 10 cases, 500 opcodes
  xlarge: 5 cases, 1000 opcodes
  xxlarge: 3 cases, 2000 opcodes

Performance Results:
┌─────────┬─────────┬─────────────┬─────────────┬──────────┬─────────────┐
│ Size    │ Length  │ Interpreter │ JIT         │ Speedup  │ JIT Benefit │
├─────────┼─────────┼─────────────┼─────────────┼──────────┼─────────────┤
│ small   │      20 │       734ns │        81ns │    9.06x │     806.2%  │
│ medium  │     100 │      2091ns │       143ns │   14.62x │    1362.2%  │
│ large   │     500 │      8703ns │       490ns │   17.76x │    1676.1%  │
│ xlarge  │    1000 │     16846ns │       914ns │   18.43x │    1743.1%  │
│ xxlarge │    2000 │     34211ns │      1607ns │   21.29x │    2029.3%  │
└─────────┴─────────┴─────────────┴─────────────┴──────────┴─────────────┘

Analysis:
  Average speedup: 16.23x
  Best speedup: 21.29x (xxlarge)
  Worst speedup: 9.06x (small)

✓ JIT shows consistent performance benefits across all test sizes
✓ JIT achieves significant speedups (>2x) on some workloads
```
