# Live on-chain anchor — receipt

A genuine, node-validated, mined transaction committing an `idattr` identity-registry root to the
BSV chain (Teranode v0.15.1 regtest, RPC `127.0.0.1:9292`), using the SCARCITY / TEA-BSV
**note-anchoring template** — **not** OP_RETURN. Performed 2026-06-02.

| Field | Value |
|---|---|
| **Anchor tx id** | `068093ae5388b190df316039b9ab56ff6bfa9d0fa6b7039b5c4f79d697840580` |
| **Committed registry root** | `700e28d345abb17b8e911893e37fcfc07a9885d8167443c2aa9c5919041d74d4` |
| **Commitment output (vout0)** | `nonstandard` — asm: `700e28d3…41d74d4 OP_DROP OP_DUP OP_HASH160 bb4d435b…59401484 OP_EQUALVERIFY OP_CHECKSIG` |
| **Carrier** | note-anchoring envelope-drop: the root rides as pushdata then `OP_DROP`, ahead of a native **P2PKH spend tail** — the output is a genuinely **spendable** possession outpoint, **no OP_RETURN** |
| **Output value** | 24.999995 BSV (spendable; the funded input minus the 500-sat fee) |
| **Funding** | a 25 BSV coinbase (`bc91eb6e…4ec47d1c`, 2,500,000,000 sat) at height 208, matured (100 confirmations) |
| **Signature** | standard BSV BIP143 / `SIGHASH_ALL\|SIGHASH_FORKID`, accepted by the node |
| **Broadcast** | `sendrawtransaction` returned the txid with no error (node accepted the non-OP_RETURN script) |
| **Mined into** | block **309** (`13be6a06…d3f7b76a`, `merkleroot 6d69a929…ed0bc6cf`, `num_tx = 2`) |

The block at height 309 has `num_tx = 2` (coinbase + this anchor transaction), i.e. the anchor was
included; the registry root is therefore committed and confirmed on-chain, with a Merkle path from
the root through the block's `merkleroot`. `getrawtransaction` confirms `vout0` contains **no**
`OP_RETURN`: the 32-byte root is carried as pushdata in a spendable locking script, per
REQ-CHAIN-0003 (state root committed by the possession transaction), REQ-CHAIN-0001/0002 (standard
P2PKH), and the absolute OP_RETURN prohibition (REQ-CHAIN-0051 / REQ-BUILD-0010).

## Why this is the SCARCITY-compliant form (not OP_RETURN)

The earlier draft used an `OP_FALSE OP_RETURN <root>` data carrier. That violates SCARCITY
REQ-CHAIN-0051 ("no OP_RETURN exists to carry one … smuggling into other fields is equally
prohibited") and REQ-BUILD-0010 (OP_RETURN is an absolute prohibition). The compliant template
commits the root **inside a spendable output** so that the commitment output is itself the P2PKH
possession outpoint — exactly the carrier built by `bsvscript::BuildEnvelopeDrop`
(`SQL/services-go/bsvscript`) and used by `SPV/03-identity/anchor`.

## Reproduce

Bring the node up — **and keep the WSL VM pinned** so it does not idle-shut-down the stack:

```powershell
# pin the WSL VM (run in background; otherwise WSL tears down the containers when idle)
wsl -u root bash -lc "sleep 3600"
# bring the regtest stack up
wsl -u root bash -lc "cd /root/teranode-quickstart && docker compose up -d"
```

Then run the `keygen` → `generatetoaddress 101 <ADDRESS>` → read the matured coinbase
(`txid = block.merkleroot` for a single-tx block; read its `value`) → `build-anchor` →
`sendrawtransaction <RAWTX>` → `generatetoaddress 1` sequence (see README). RPC needs an explicit
HTTP Basic `Authorization` header (`teranode:regtestsecret`); block height updates asynchronously
(`getblockchaininfo.blocks`, not `getblockcount`). The registry root may be any `idattr` anchor
root, e.g. the output of `idattr demo` / `GET /anchor` on the registry service.
