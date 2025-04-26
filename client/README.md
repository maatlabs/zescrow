# Zescrow Client

## Create chain-specific (e.g., Solana) escrow

```sh
zescrow-cli create \
    --chain solana \
    --config chain_config.json \
    --params escrow_params.json \
    --outfile escrow_metadata.json
```

## Release escrow

```sh
zescrow-cli release --metadata escrow_metadata.json
```

## Refund escrow

```sh
zescrow-cli refund --metadata escrow_metadata.json
```
