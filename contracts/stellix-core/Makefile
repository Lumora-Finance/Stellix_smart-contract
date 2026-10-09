SHELL := /bin/bash

NETWORK ?= testnet
SOURCE ?= stellix-deployer
WASM := target/wasm32v1-none/release/stellix_core.wasm

.PHONY: all build test lint fmt clean deploy init

all: build

## Build the optimized Soroban WASM.
build:
	stellar contract build

## Run the unit test suite.
test:
	cargo test -p stellix-core

## Lint all targets.
lint:
	cargo clippy --all-targets -- -D warnings

## Format Rust sources.
fmt:
	cargo fmt --all

## Remove build artifacts.
clean:
	cargo clean

## Deploy the built WASM to a network. Override NETWORK/SOURCE as needed.
deploy: build
	stellar contract deploy \
		--wasm $(WASM) \
		--source $(SOURCE) \
		--network $(NETWORK)

## Initialize a deployed contract. Requires ADMIN and CONTRACT env vars.
init:
	@test -n "$(CONTRACT)" || (echo "CONTRACT is required" && exit 1)
	@test -n "$(ADMIN)" || (echo "ADMIN is required" && exit 1)
	stellar contract invoke \
		--id $(CONTRACT) \
		--source $(SOURCE) \
		--network $(NETWORK) \
		-- initialize --admin $(ADMIN)
