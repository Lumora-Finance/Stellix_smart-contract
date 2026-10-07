# Stellix Smart Contract

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-7B2FBE)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-2021-orange)](https://www.rust-lang.org)

A Soroban smart contract powering the **Stellix** decentralized finance platform on the Stellar network. Stellix is a modern DeFi invoicing and payment management protocol built on Stellar's Soroban smart contract platform.

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Build](#build)
- [Testing](#testing)
- [Deployment](#deployment)
- [Contract Interface](#contract-interface)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

The Stellix smart contract is written in Rust and compiled to WebAssembly (WASM) for deployment on the Stellar Soroban platform. It provides the on-chain logic for Stellix's financial operations, including invoice tracking, payment state management, and on-chain counters for protocol metrics.

---

## Features

- **On-chain state management** via Soroban instance storage
- **Counter tracking** for protocol-level metrics
- **No-std Rust** — minimal footprint, optimized for WASM
- **Fully tested** with Soroban's native test environment
- **Optimized release profile** for minimal WASM binary size

---

## Architecture

```
stellix_smartcontract/
├── src/
│   └── lib.rs          # Core contract logic
├── Cargo.toml           # Rust package manifest & dependencies
├── .gitignore           # Ignored build artifacts
└── README.md            # This file
```

The contract uses:
- **`soroban-sdk`** for Stellar's smart contract primitives
- **Instance storage** for persistent on-chain state
- **WASM target** (`wasm32-unknown-unknown`) for Soroban deployment

---

## Prerequisites

Make sure you have the following installed:

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable (2021 edition) | `curl https://sh.rustup.rs -sSf \| sh` |
| WASM target | wasm32-unknown-unknown | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | latest | `cargo install --locked stellar-cli` |

Verify your setup:

```bash
rustc --version
stellar --version
```

---

## Installation

Clone the repository and navigate into the contract directory:

```bash
git clone https://github.com/Lumora-Finance/Stellix_smart-contract.git
cd Stellix_smart-contract
```

---

## Build

Build the contract to a WASM binary using the Stellar CLI:

```bash
stellar contract build
```

The compiled binary will be output to:

```
target/wasm32-unknown-unknown/release/stellix_smartcontract.wasm
```

To optimize the binary further (optional):

```bash
stellar contract optimize \
  --wasm target/wasm32-unknown-unknown/release/stellix_smartcontract.wasm
```

---

## Testing

Run the full test suite using Cargo:

```bash
cargo test
```

Tests use Soroban's built-in test environment, which simulates the blockchain without needing a live network.

---

## Deployment

### Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellix_smartcontract.wasm \
  --source <YOUR_ACCOUNT> \
  --network testnet
```

### Mainnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellix_smartcontract.wasm \
  --source <YOUR_ACCOUNT> \
  --network mainnet
```

> **Note:** Make sure your account is funded with XLM before deploying. Use [Stellar Laboratory](https://laboratory.stellar.org) or the [Friendbot faucet](https://friendbot.stellar.org) for testnet funding.

---

## Contract Interface

### `increment() → u32`

Increments the on-chain counter by 1 and returns the new value.

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_ACCOUNT> \
  --network testnet \
  -- increment
```

### `get() → u32`

Returns the current value of the counter without modifying state.

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_ACCOUNT> \
  --network testnet \
  -- get
```

---

## Project Structure

```
src/
└── lib.rs
    ├── StellixContract        # Main contract struct
    ├── increment()            # Mutating: increments counter
    ├── get()                  # Read-only: returns current counter
    └── test module            # Unit tests using soroban-sdk testutils
```

---

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Commit your changes: `git commit -m "feat: add your feature"`
4. Push to the branch: `git push origin feature/your-feature`
5. Open a Pull Request

Please make sure all tests pass before submitting a PR.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

Built with ❤️ by the [Lumora Finance](https://github.com/Lumora-Finance) team.
