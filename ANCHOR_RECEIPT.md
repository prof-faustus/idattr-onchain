# Live on-chain anchor — receipt

A genuine, node-validated, mined transaction committing an `idattr` identity-registry root to the
BSV chain (Teranode v0.15.1 regtest, RPC `127.0.0.1:9292`). Performed 2026-06-02.

| Field | Value |
|---|---|
| **Anchor tx id** | `1a6b2af70d01a7e07a0dbdf4394258f1d79ff37f74df9ee05be5290e9ff87c1e` |
| **Committed registry root** | `700e28d345abb17b8e911893e37fcfc07a9885d8167443c2aa9c5919041d74d4` |
| **Commitment output (vout0)** | `nulldata` — asm: `0 OP_RETURN 700e28d3…41d74d4` |
| **Funding** | a 50 BSV coinbase (`4f0f897e…946b090a`, 5,000,000,000 sat) at height 106 |
| **Signature** | standard BSV BIP143 / `SIGHASH_ALL\|SIGHASH_FORKID`, accepted by the node |
| **Broadcast** | `sendrawtransaction` returned the txid with no error (node accepted) |
| **Mined into** | block **207** (`merkleroot 7ebc039d61351648843a159f663d166d93c3a4eebeea58ce1fbf3a6fb61878a7`, `num_tx = 2`) |
| **Change (vout1)** | 49.999995 BSV (500 sat fee) back to the spender |

The surrounding blocks 205/206 have `num_tx = 1` (coinbase only); block 207 has `num_tx = 2`,
i.e. the anchor transaction was included. The registry root is therefore committed and confirmed
on-chain, with a Merkle path from the root through the block's `merkleroot`.

## Reproduce

Bring the node up (`cd /root/teranode-quickstart && docker compose up -d`), then run the
`keygen` → `generatetoaddress 101` → read coinbase → `build-anchor` → `sendrawtransaction` →
`generatetoaddress 1` sequence (see README). The registry root may be any `idattr` anchor root,
e.g. the output of `idattr demo` / `GET /anchor` on the registry service.
