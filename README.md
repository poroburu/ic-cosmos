# IC-cosmos Gateway

[![Internet Computer portal](https://img.shields.io/badge/InternetComputer-grey?logo=internet%20computer&style=for-the-badge)](https://internetcomputer.org)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](LICENSE)
[![Tests Status](https://img.shields.io/github/actions/workflow/status/mfactory-lab/ic-cosmos/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)

> #### Interact with [cosmos](https://cosmos.com) from the [Internet Computer](https://internetcomputer.org/).

> [!Note]
> This project is a work in progress and is not yet ready for production use. We are happy to answer questions if they are raised as issues in this GitHub repo.

## Overview

**IC-cosmos** is a solution that connects the [Internet Computer](https://internetcomputer.org/) with [cosmos](https://cosmos.com/). It allows developers to build decentralized applications (dApps) on the Internet Computer with functionality comparable to traditional cosmos dApps. This integration combines the capabilities of both blockchain networks, making it easier to develop cross-chain applications and expand the possibilities for decentralized solutions.

## Quick start

Add the following configuration to your `dfx.json` file (replace the `ic` principal with any option from the list of available canisters):

```json
{
  "canisters": {
    "cosmos_rpc": {
      "type": "custom",
      "candid": "https://github.com/mfactory-lab/ic-cosmos/blob/main/src/ic-cosmos-rpc/ic-cosmos-rpc.did",
      "wasm": "https://github.com/mfactory-lab/ic-cosmos/blob/main/ic-cosmos-rpc.wasm.gz",
      "init_arg": "(record {})"
    },
    "cosmos_wallet": {
      "type": "custom",
      "candid": "https://github.com/mfactory-lab/ic-cosmos/blob/main/src/ic-cosmos-wallet/ic-cosmos-wallet.did",
      "wasm": "https://github.com/mfactory-lab/ic-cosmos/blob/main/ic-cosmos-wallet.wasm.gz",
      "init_arg": "(record {})"
    }
  }
}
```

## Running the project locally

### Requirements

Make sure you have the following installed:

- [Rust](https://www.rust-lang.org/learn/get-started)
- [Docker](https://www.docker.com/get-started/) (optional for [reproducible builds](#reproducible-builds))
- [PocketIC](https://github.com/dfinity/pocketic) (optional for testing)
- [DFINITY SDK](https://sdk.dfinity.org/docs/quickstart/local-quickstart.html)

### Building the code

Start a local replica listening on port 4943:

```bash
# Start a local replica
dfx start --clean --host 127.0.0.1:4943
```

Build and deploy canisters:

```bash
# Deploy the `cosmos_rpc` canister locally
dfx deploy cosmos_rpc --argument '(record {})'

# Deploy the `cosmos_wallet` canister locally
dfx deploy cosmos_wallet --argument "(record { cos_canister = opt principal \"`dfx canister id cosmos_rpc`\"; schnorr_key = null })"
```

All the canisters will be deployed to the local network with their fixed canister IDs.

Once the build and deployment are complete, your application will be accessible at:

```
http://localhost:4943?canisterId={asset_canister_id}
```

Replace `{asset_canister_id}` with the actual canister's ID generated during deployment.

## Examples

Use the cosmos mainnet cluster:

```bash
dfx canister call cosmos_rpc cos_getHealth '(variant{Mainnet},null)' --wallet $(dfx identity get-wallet)
```

Use the cosmos devnet cluster:

```bash
dfx canister call cosmos_rpc cos_getHealth '(variant{Devnet},null)' --wallet $(dfx identity get-wallet)
```

Use a single custom RPC:

```bash
dfx canister call cosmos_rpc cos_getHealth '(variant{Custom=vec{record{network="https://mainnet.helius-rpc.com/"}}},null)' --wallet $(dfx identity get-wallet)
```

Use multiple custom RPCs:

```bash
dfx canister call cosmos_rpc cos_getHealth '(variant{Custom=vec{record{network="mainnet"},record{network="https://mainnet.helius-rpc.com/"}}},null)' --wallet $(dfx identity get-wallet)
```

Use a single RPC provider (predefined providers: mainnet|m, devnet|d, testnet|t):

```bash 
dfx canister call cosmos_rpc cos_getHealth '(variant{Provider=vec{"mainnet"}},null)' --wallet $(dfx identity get-wallet)
```

## Components

### [RPC Canister](./src/ic-cosmos-rpc)

The **RPC Canister** enables communication with the cosmos blockchain, using [HTTPS outcalls](https://internetcomputer.org/https-outcalls) to transmit raw transactions and messages via on-chain APIs of [cosmos JSON RPC](https://cosmos.com/docs/rpc) providers, for example, [Helius](https://www.helius.dev/) or [Quicknode](https://www.quicknode.com/).

Key functionalities include:

1. Retrieving cosmos-specific data, such as block details, account information, node statistics, etc.
2. Managing cosmos RPC providers, including registration, updates, and provider configurations.
3. Calculating and managing the cost of RPC requests.

[//]: # (The RPC Canister runs on the 34-node [fiduciary subnet]&#40;https://internetcomputer.org/docs/current/references/subnets/subnet-types#fiduciary-subnets&#41;)

[//]: # (with the following principal: [bd3sg-teaaa-aaaaa-qaaba-cai]&#40;https://dashboard.internetcomputer.org/canister/bd3sg-teaaa-aaaaa-qaaba-cai&#41;.)

### [Wallet Canister](./src/ic-cosmos-wallet)

The **Wallet Canister** is used for managing addresses and for securely signing transactions/messages for the cosmos blockchain using the [threshold Schnorr API](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr).

Key functionalities include:

1. Generating a cosmos public key (Ed25519) for a user on the Internet Computer (ICP).
2. Signing messages using distributed keys based on the `Threshold Schnorr` protocol.
3. Signing and sending raw transactions to the cosmos blockchain via the [RPC Canister](#rpc-canister).

### [IC-cosmos](./src/ic-cosmos)

A Rust library that provides the necessary tools for integrating cosmos with ICP canisters.

## Access control

IC-cosmos stores a list of registered cosmos JSON RPC providers, to which transactions and messages can be submitted. Access to the list is controlled by admin(s) who can assign managers with specific rights to add, remove, and update cosmos JSON RPC providers.

## Reproducible builds

IC-cosmos supports [reproducible builds](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/test/reproducible-builds):

1. Ensure [Docker](https://www.docker.com/get-started/) is installed on your machine.
2. Run `./scripts/docker-build --rpc` in your terminal.
3. Run `sha256sum ic-cosmos-rpc.wasm.gz` on the generated file to view the SHA-256 hash.

Compare the generated SHA-256 hash with the hash provided in the repository to verify the build's integrity.

## Learn more

- [Candid Interface](https://github.com/mfactory-lab/ic-cosmos/blob/main/src/ic-cosmos-rpc/ic-cosmos-rpc.did)
- [cosmos JSON RPC API](https://cosmos.com/docs/rpc)
- [Internet Computer Developer Docs](https://internetcomputer.org/docs/current/developer-docs/)
- [DFINITY SDK Documentation](https://sdk.dfinity.org/docs/)
- [Internet Computer HTTPS Outcalls](https://internetcomputer.org/https-outcalls)

## Contributing

Contributions are welcome! Please check out the [contributor guidelines](https://github.com/mfactory-lab/ic-cosmos/blob/main/.github/CONTRIBUTING.md) for more information.

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).
