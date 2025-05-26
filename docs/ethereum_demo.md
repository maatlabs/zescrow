# End-to-End Flow for Ethereum Escrows

1. Build and deploy the on-chain Zescrow adapter for Ethereum (i.e., `EscrowFactory` + `Verifier`). Take note of the contract addresses.

2. Edit the [`templates/escrow_params.json`](/templates/escrow_params.json) file to specify the parameters of your escrow.

When in doubt, please check the definition of `EscrowParams` in the [`core` interface](/core/src/interface.rs), which provides the full context for what's expected.
