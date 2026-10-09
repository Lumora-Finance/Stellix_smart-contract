#!/usr/bin/env bash
#
# Build and deploy Stellix Core to a Stellar network, then initialize it.
#
# Usage:
#   NETWORK=testnet SOURCE=stellix-deployer ./scripts/deploy.sh
#
# Environment:
#   NETWORK  Target network (default: testnet)
#   SOURCE   Stellar CLI identity used to sign (default: stellix-deployer)
#   ADMIN    Admin address stored on-chain (default: address of SOURCE)
#   WASM     Path to the built wasm (default: target/wasm32v1-none/release/stellix_core.wasm)
set -euo pipefail

NETWORK="${NETWORK:-testnet}"
SOURCE="${SOURCE:-stellix-deployer}"
WASM="${WASM:-target/wasm32v1-none/release/stellix_core.wasm}"

if [[ -z "${ADMIN:-}" ]]; then
  ADMIN="$(stellar keys address "${SOURCE}")"
fi

echo "==> Building contract"
stellar contract build

echo "==> Deploying to ${NETWORK} as ${SOURCE}"
CONTRACT_ID="$(stellar contract deploy \
  --wasm "${WASM}" \
  --source "${SOURCE}" \
  --network "${NETWORK}" | tail -n 1)"
echo "    contract id: ${CONTRACT_ID}"

echo "==> Initializing with admin ${ADMIN}"
stellar contract invoke \
  --id "${CONTRACT_ID}" \
  --source "${SOURCE}" \
  --network "${NETWORK}" \
  -- initialize --admin "${ADMIN}"

echo
echo "✅ Stellix Core deployed and initialized"
echo "   CONTRACT_ID=${CONTRACT_ID}"
