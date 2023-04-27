# Lido Terra Contracts

This monorepository contains the source code for the smart contracts implementing bAsset Protocol on the [Terra](https://terra.money) blockchain.

You can find information about the architecture, usage, and function of the smart contracts on the official documentation [site](https://lidofinance.github.io/terra-docs/).


## Contracts
| Contract                                            | Reference                                              | Description                                                                                                                        |
| --------------------------------------------------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| [`basset_sei_hub`](https://github.com/KryptoniteDAO/krp-staking-contracts/tree/master/contracts/basset_sei_hub)|[doc](https://lidofinance.github.io/terra-docs/contracts/hub)| Manages minted bsei and bonded sei
| [`basset_sei_reward`](https://github.com/KryptoniteDAO/krp-staking-contracts/tree/master/contracts/basset_sei_reward)|[doc](https://lidofinance.github.io/terra-docs/contracts/reward)|Manages the distribution of delegation rewards
| [`basset_sei_token_bsei`](https://github.com/KryptoniteDAO/krp-staking-contracts/tree/master/contracts/basset_sei_token_bsei)| [doc](https://lidofinance.github.io/terra-docs/contracts/stLuna_and_bLuna)|CW20 compliance
| [`basset_sei_rewards_dispatcher`](https://github.com/KryptoniteDAO/krp-staking-contracts/tree/master/contracts/basset_sei_rewards_dispatcher)| [doc](https://github.com/KryptoniteDAO/krp-staking-contracts/tree/master/contracts/rewards_dispatcher)|Accumulates the rewards from Hub's delegations and manages the rewards
| [`basset_sei_token_stsei`](https://github.com/KryptoniteDAO/krp-staking-contracts/tree/master/contracts/basset_sei_token_stsei)| [doc](https://lidofinance.github.io/terra-docs/contracts/stLuna_and_bLuna)|CW20 compliance for stsei
| [`basset_sei_validators_registry`](https://github.com/KryptoniteDAO/krp-staking-contracts/tree/master/contracts/basset_sei_validators_registry)| [doc](https://lidofinance.github.io/terra-docs/contracts/validators_registry)|Approved validators whitelist

## Development

### Environment Setup

- Rust v1.55.0+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit tests embedded within the contract source directories. You can run:

```sh
cargo test unit-test
cargo test integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.11.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

## Documentation

Check you the documentation at https://lidofinance.github.io/terra-docs/.

## License

Copyright 2021 Lido

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
