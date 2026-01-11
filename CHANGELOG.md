# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-11

### Added

#### New Crate: `zescrow-prover`

- Extracted RISC Zero zkVM prover into a standalone crate for build optimization
- ZK proving is now opt-in via `--features prover` flag
- Users without RISC Zero toolchain can build and use timelock-only escrows

#### Core Library (`zescrow-core`)

- `#[non_exhaustive]` on all public error enums for future extensibility
- `#[must_use]` attributes on verification methods
- Module-level documentation for all public modules
- Security considerations in cryptographic condition documentation
- Environment variable expansion (`${VAR_NAME}`) in JSON configuration files
- New tests: escrow error paths, interface module, serialization roundtrips (24 -> 46 tests)

#### Client (`zescrow-client`)

- `dotenvy` integration for `.env` file loading
- Structured error context via `ClientError::ethereum()` and `ClientError::solana()` helpers
- Comprehensive documentation for `Agent` trait and all public APIs

#### Solana Program (`escrow`)

- Devnet program configuration in `Anchor.toml`
- Package metadata (description, license, repository)

#### Ethereum Contract

- `nextEscrowId()` getter for next escrow ID
- `escrowCount()` getter for total escrows created
- Sepolia network configuration in `hardhat.config.ts`

#### Deployment & CI

- Consolidated `/deploy/` directory with Solana devnet and Ethereum Sepolia scripts
- `.env.template` for environment variable configuration
- `cargo-deny` integration for dependency auditing
- Development commands section in README
- Optimized build workflow: debug builds for `create`, `cancel`, `generate`; release only for `finish` with prover
- `RUST_LOG=info` default in `.env.template` for CLI output visibility

### Changed

#### Core Library (`zescrow-core`)

- Refactored `ID::from_str` to use functional combinators
- Refactored `Escrow::execute` with functional state transitions
- Refactored `Threshold::verify` with iterator combinators
- Refactored `Asset::validate()` with functional match arms
- Improved `format_amount()` error handling

#### Client (`zescrow-client`)

- Unified `ClientError` hierarchy (flattened from separate `AgentError`)
- Extracted helper functions in Ethereum agent: `load_contract_abi()`, `create_contract_instance()`, `extract_escrow_id()`
- Extracted helper functions in Solana agent: `build_create_instruction()`, `build_finish_instruction()`, `derive_escrow_pda()`
- Refactored prover module with structured logging via `tracing` spans

#### Solana Program (`escrow`)

- Updated Anchor from 0.31.1 to 0.32.1

#### CI/CD

- Updated GitHub Actions: `actions/checkout@v4`, `dtolnay/rust-toolchain`, `actions/cache@v4`
- Added `RISC0_SKIP_BUILD=1` for faster CI builds

### Fixed

- Typo in `AssetError::InvalidId` ("inalid" → "invalid")
- Unused import `ed25519_dalek::Verifier` in `secp256k1.rs`
- Rustdoc HTML escaping for `Vec<u8>` in serde module
- Hex identity parsing now handles optional `0x` prefix correctly

### Removed

- Legacy `/templates/` directory (consolidated into `/deploy/`)

## [0.1.0] - 2024-12-15

Initial release.

### Added

- Chain-agnostic escrow core library with cryptographic conditions
- RISC Zero zkVM integration for zero-knowledge proofs
- Solana Anchor program with XRPL-style timelock semantics
- Ethereum Solidity contract with EscrowFactory pattern
- CLI client for cross-chain escrow operations
- Support for hashlock, Ed25519, Secp256k1, and threshold conditions
