# safe-gasless-api

A Rust HTTP service that creates [Gnosis Safe](https://safe.global) wallets deterministically using `CREATE2`, and relays user transactions without the user paying gas.

## Why it exists

Onboarding to self-custody wallets is gated by gas. For a user who has never held ETH, the first transaction they need to sign is the one that funds their wallet — but they can't sign it without ETH. A gasless relay closes that loop: the backend deploys the user's Safe, the backend pays gas, and the user signs only the inner intent.

This is a minimal reference implementation: one binary, one endpoint per operation, no queueing layer.

## Architecture

Actix-web service in front of an Ethereum RPC endpoint. Core flows:

- **`POST /safe`** — given an owner address and salt nonce, compute the counterfactual Safe address via `CREATE2` and return it. No on-chain action.
- **`POST /safe/deploy`** — deploy the Safe at its counterfactual address. Backend pays gas.
- **`POST /safe/exec`** — relay a signed `execTransaction` through the Safe. User signature validates the inner call; backend wraps it in the outer meta-transaction and pays gas.

Chain interaction via `ethers-rs`. ABI encoding/decoding generated from the Gnosis Safe factory and proxy contracts.

## Build and run

```bash
cargo run --release
```

Requires a running Ethereum JSON-RPC endpoint (local anvil / hardhat / geth in dev, any provider in prod).

## Configuration

Copy `.env.example` to `.env` and fill in:

```
RPC_URL=http://localhost:8545
BACKEND_PRIVATE_KEY=0x<hex>               # relayer, must hold ETH
MASTER_COPY_CONTRACT_ADDRESS=<safe-master-copy>
FALLBACK_ADDRESS=<fallback-handler>
SALT_NONCE=0xcafebabe
```

Do not commit your real `.env`. The `BACKEND_PRIVATE_KEY` controls the relayer wallet.

## Example

```bash
# 1. Compute counterfactual address
curl -X POST localhost:8080/safe \
  -H 'content-type: application/json' \
  -d '{"owner":"0xabc...","saltNonce":"0x01"}'

# {"address":"0xdef..."}

# 2. Deploy at that address
curl -X POST localhost:8080/safe/deploy \
  -H 'content-type: application/json' \
  -d '{"owner":"0xabc...","saltNonce":"0x01"}'

# {"txHash":"0x..."}
```

## What it doesn't do

- Not a production relayer. No rate limiting, no nonce management under load, no backoff on mempool congestion.
- Single-signer Safes only. No threshold-signature flows.
- No meta-transaction standard beyond Gnosis Safe's own `execTransaction`. No ERC-4337.
- No front-running protection beyond the backend wallet's nonce ordering.

## License

MIT.
