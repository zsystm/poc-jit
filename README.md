```sh
docker buildx build \
  --platform linux/amd64 \
  -t evm_poc_image \
  --load \
  .

docker run -it --rm evm_poc_image /bin/bash

cargo run --release
```
