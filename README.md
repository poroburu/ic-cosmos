# IC-Solana Gateway for the Internet Computer

[![Internet Computer portal](https://img.shields.io/badge/InternetComputer-grey?logo=internet%20computer&style=for-the-badge)](https://internetcomputer.org)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](LICENSE)
[![Tests Status](https://img.shields.io/github/actions/workflow/status/mfactory-lab/ic-solana/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)

> #### Interact with [Solana](https://solana.com) from the [Internet Computer](https://internetcomputer.org/).

> This project is a work in progress, and is not yet ready for production use. We are happy to answer questions if they are raised as issues in this github repo.

## Overview

**IC-Solana** is a seamless integration solution that bridges the [Solana](https://solana.com/) blockchain
with the [Internet Computer](https://internetcomputer.org/).
It empowers developers to build decentralized applications (dApps)
on the Internet Computer with functionality comparable to traditional Solana dApps.

This integration combines the capabilities of both platforms,
making it easier to develop cross-chain applications and expand the possibilities for decentralized solutions.

## Quick start

Add the following configuration to your `dfx.json` file (replace the
`ic` principal with any option from the list of available canisters above):

```json
{
  "canisters": {
    "solana_rpc": {
      "type": "custom",
      "candid": "https://github.com/mfactory-lab/ic-solana/blob/main/src/ic-solana-rpc/ic-solana-rpc.did",
      "wasm": "https://github.com/mfactory-lab/ic-solana/blob/main/ic-solana-rpc.wasm.gz",
      "init_arg": "(record {})"
    },
    "solana_wallet": {
      "type": "custom",
      "candid": "https://github.com/mfactory-lab/ic-solana/blob/main/src/ic-solana-wallet/ic-solana-wallet.did",
      "wasm": "https://github.com/mfactory-lab/ic-solana/blob/main/ic-solana-wallet.wasm.gz",
      "init_arg": "(record {})"
    }
  }
}
```

## Running the project locally

### Requirements

Please make sure you have the following installed:

- [Rust](https://www.rust-lang.org/learn/get-started)
- [Docker](https://www.docker.com/get-started/)
- [DFINITY SDK](https://sdk.dfinity.org/docs/quickstart/local-quickstart.html)

### Building the Code

Start a local replica listening on port 4943:

```bash
# Start the local replica
dfx start --clean --host 127.0.0.1:4943
```

Build and deploy canisters:

```bash
# Locally deploy the `solana_rpc` canister
dfx deploy solana_rpc --argument '(record {})'

# Locally deploy the `solana_wallet` canister
dfx deploy solana_wallet --argument '(record {})'
```

All the canisters will be deployed to the `local` network with their fixed canister ids.

Once the build and deployment are complete, your application will be accessible at:

```
http://localhost:4943?canisterId={asset_canister_id}
```

Replace `{asset_canister_id}` with the actual canister ID generated during deployment.

## Examples

JSON-RPC (mainnet)

```bash
# Use Solana Mainnet
dfx canister call solana_rpc sol_getLatestBlockhash '(variant {Mainnet})' --wallet $(dfx identity get-wallet --ic) --with-cycles 1000000000

# Use Custom RPC
dfx canister call solana_rpc sol_getLatestBlockhash '(variant {Custom=record {url="https://example-rpc.com"}})' --wallet $(dfx identity get-wallet --ic) --with-cycles 1000000000
```

## Resources

### [RPC Canister](./src/ic-solana-rpc)

The **RPC Canister** enables seamless communication with the Solana blockchain
by utilizing [HTTPS outcalls](https://internetcomputer.org/https-outcalls)
to transmit raw transactions and messages via the on-chain APIs
provided by [Solana JSON RPC](https://solana.com/docs/rpc) services,
such as [Helius](https://www.helius.dev/) or [Quicknode](https://www.quicknode.com/).

Key functionalities include:

1. Retrieving Solana-specific data, such as block details, account information, node statistics, and more.
2. Managing Solana RPC providers, including registration, updates, and provider configurations.
3. Calculating and managing the cost of RPC requests for efficient resource utilization.

[//]: # (The RPC canister runs on the 34-node [fiduciary subnet]&#40;https://internetcomputer.org/docs/current/references/subnets/subnet-types#fiduciary-subnets&#41;)

[//]: # (with the following principal: [bd3sg-teaaa-aaaaa-qaaba-cai]&#40;https://dashboard.internetcomputer.org/canister/bd3sg-teaaa-aaaaa-qaaba-cai&#41;.)

### [Wallet Canister](./src/ic-solana-wallet)

The **Wallet Canister** serves as the core component for managing addresses
and signing transactions or messages on the Solana blockchain.
It leverages the [Threshold Schnorr API](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr) to perform secure and distributed signing of messages and transactions.

Key functionalities include:

1. Generating a Solana public key (Ed25519) for a user on the Internet Computer (ICP).
2. Signing messages using distributed keys based on the Threshold Schnorr protocol.
3. Signing and sending raw transactions to the Solana blockchain via the [RPC Canister](#rpc-canister).

### [IC-Solana](./src/ic-solana)

Rust library that provides the necessary tools for integrating with Solana into IC canisters.

### Access control

The Solana RPC canister stores a list of registered Solana JSON RPC providers, to which transactions and messages can be submitted.
Access to the list is controlled by admin(s) who can assign managers with specific rights to add, remove, and update Solana JSON RPC providers.

### Reproducible builds

The SOLANA RPC canister supports [reproducible builds](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/test/reproducible-builds):

1. Ensure [Docker](https://www.docker.com/get-started/) is installed on your machine.
2. Run `./scripts/docker-build --rpc` in your terminal.
3. Run `sha256sum ic-solana-rpc.wasm.gz` on the generated file to view the SHA-256 hash.

Compare the generated SHA-256 hash with the hash provided in the repository to verify the build's integrity.

## Learn more

To learn more about the SOLANA RPC Canister and its integration with Solana and ICP, explore the following resources:

- [Candid Interface](https://github.com/mfactory-lab/ic-solana/blob/main/src/ic-solana-rpc/ic-solana-rpc.did)
- [Solana JSON RPC API](https://solana.com/docs/rpc)
- [Internet Computer Developer Docs](https://internetcomputer.org/docs/current/developer-docs/)
- [DFINITY SDK Documentation](https://sdk.dfinity.org/docs/)
- [Internet Computer HTTPS Outcalls](https://internetcomputer.org/https-outcalls)

## Contributing

Contributions are welcome!
Please check out the [contributor guidelines](https://github.com/mfactory-lab/ic-solana/blob/main/.github/CONTRIBUTING.md) for more information.

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).
