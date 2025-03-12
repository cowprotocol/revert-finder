### Description
Tool to debug what priority fee a transaction should have at least to make it onchain. It does that by finding the highest index in a block where the transaction does not revert. To ensure the transaction would have made it onchain a priority fee that beats the "rival" transaction, that caused the target transaction to revert, would have been necessary.

This tool relies on an RPC that supports the `debug_traceCallMany` method. For example this is supported by [reth](https://github.com/paradigmxyz/reth) or [geth](https://github.com/ethereum/go-ethereum).
Also this tool currently does not work for transactions that only work when applying access lists. This mean primarly CoW protocol settlement that pay out native ETH to smart contract addresses.
However, adding support for this only requires the access list to be passed via a CLI argument.

### Usage
```
sage: revert-finder --rpc <RPC> --block <BLOCK> --to <TO> --from <FROM> --calldata <CALLDATA>

Options:
  -r, --rpc <RPC>            RPC to simulate calls with. Needs to support `debug_traceCallMany` [env: RPC=]
  -b, --block <BLOCK>        Block in which index for last successful simulation should be found [env: BLOCK=]
  -t, --to <TO>              Contract to which the transaction should be sent [env: TO=]
  -f, --from <FROM>          Address that would have sent the transaction [env: FROM=]
  -c, --calldata <CALLDATA>  Calldata of the transaction (hex string with or without `0x` prefix) [env: CALLDATA=]
  -h, --help                 Print help
  -V, --version              Print versio
```

This repo also includes a devcontainer that can be used with VSCode or code spaces.
