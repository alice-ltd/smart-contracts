# aliceUST Token

[![codecov](https://codecov.io/gh/Alice-Ltd/smart-contract/branch/main/graph/badge.svg?token=ER41USPZBX)](https://codecov.io/gh/Alice-Ltd/smart-contract)

See `docs` folder for detailed documentation.

Reference template: https://github.com/CosmWasm/cosmwasm-template

## Unit tests

To perform unit tests, first make sure you have `cargo`, `rustup`, `rustc`, etc installed. Then,

``cargo test``

## End-to-end tests

```
cd utils
npm install
npm test
```

## Deploy

Follow [instructions on Terra docs](https://docs.terra.money/contracts/tutorial/interacting.html#requirements) to deploy

```bash
bash -c 'docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.4'
```