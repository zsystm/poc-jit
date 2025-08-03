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

## Why is JIT So Much Faster?

Both the interpreter and JIT are compiled to machine code, so why does JIT achieve 9-21x speedup? Here are the key reasons:

### 1. **Elimination of Instruction Dispatch Overhead**
- **Interpreter**: For each bytecode instruction, it must:
  1. Read the opcode from memory
  2. Jump to a lookup table or switch statement 
  3. Execute the corresponding handler function
  4. Return to the main loop
  5. Increment the program counter
  6. Repeat for the next instruction

- **JIT**: Compiles bytecode directly into a sequence of machine instructions with no dispatch overhead. A sequence like `PUSH 5, PUSH 3, ADD` becomes three consecutive machine instructions.

### 2. **Direct Memory Access vs. Abstraction Layers**
- **Interpreter**: Uses Rust's `HashMap` and `Vec` with bounds checking, memory allocation, and hash computation for storage operations.
- **JIT**: Uses direct memory access with simple pointer arithmetic (`[rbx + key * 8]` for storage), eliminating all abstraction overhead.

### 3. **Reduced Function Call Overhead**
- **Interpreter**: Each operation involves function calls to methods like `stack.push()`, `stack.pop()`, `memory.insert()`, `memory.get()`.
- **JIT**: Operations compile to direct CPU instructions like `push rax`, `pop rdx`, `add rax, rdx`.

### 4. **Better CPU Pipeline Utilization**
- **Interpreter**: Frequent branches and indirect jumps (switch statements) can cause CPU pipeline stalls and branch mispredictions.
- **JIT**: Generates linear sequences of instructions that flow naturally through the CPU pipeline.

### 5. **Specialized Code Generation**
- **Interpreter**: Generic code must handle all possible edge cases and maintain full Rust safety guarantees.
- **JIT**: Generates specialized assembly code tailored to the specific bytecode sequence being executed.

**Real-world analogy**: Think of the interpreter as a translator who reads each sentence in a foreign language, looks up each word in a dictionary, and then speaks the translation. The JIT is like learning the foreign language fluently - you can understand and respond directly without the lookup overhead.
