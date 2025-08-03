```sh
docker buildx build \
  --platform linux/amd64 \
  -t evm_poc_image \
  --load \
  .

docker run -it --rm evm_poc_image /bin/bash

cargo run --release
```

## Running experiments

Running `cargo run` generates random bytecode sequences exercising opcodes like
`PUSH`, `SLOAD`, `SSTORE`, `ADD`, `SUB` and compares an interpreter against the
JIT implementation. Results, including the executed bytecode and timing
information, are written to files under `reports/`.
