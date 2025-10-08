## rust_test

A small Rust crate with Solana dev utilities. It exposes a set of test functions (under `src/lib.rs` -> `#[cfg(test)]`) to help you quickly:

- Generate a new keypair (keygen)
- Request a devnet airdrop (claim_airdrop)
- Transfer SOL (transfer_sol)
- Drain a wallet balance (empty_wallet)
- Submit a Turbin3 prerequisite rust version (submit_rs)
- Convert private keys between Base58 and JSON byte array formats

### What it does

# It depend on the situation of the system on focus. 

This crate uses `solana-client`, `solana-sdk`, and `solana-system-interface` to connect to Solana devnet via a predefined RPC URL, construct transactions/instructions, and sign/send them with local keypairs loaded from JSON files (e.g., `dev-wallet.json`, `turbine-wallet.json`).



