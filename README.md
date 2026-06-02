# idattr-onchain

Anchor an identity-registry root **on the BSV chain**. Builds and signs a standard BSV transaction
(BIP143 / SIGHASH_FORKID) that spends a funded P2PKH output and commits the 32-byte registry/anchor
root through the SCARCITY / TEA-BSV **note-anchoring template** — never OP_RETURN — for broadcast to
a Teranode regtest node via `sendrawtransaction`.

It reuses the audited BSV primitives from `overlay-broadcast` (path deps): `bsv` (tx build, P2PKH,
`sighash`, serialize) and `ckd` (k256 ECDSA `sign_prehash_der`). It is a **separate workspace** from
the self-contained `identity-attribution` system precisely because it depends on the sibling repo.

## Use

```powershell
# 1) a key + its regtest P2PKH address (mine coins to ADDRESS)
idattr-onchain keygen
# -> PRIVKEY .. / PUBKEY .. / H160 .. / ADDRESS m...

# (mine 101 blocks to ADDRESS so a coinbase matures; read its txid = block.merkleroot and value)

# 2) build the signed anchor tx
idattr-onchain build-anchor --privkey <hex> --pubkey <hex> \
  --coinbase-txid <txid> --vout 0 --value <sats> \
  --anchor-root <32-byte-hex> [--fee 500]
# -> TXID .. / RAWTX <hex>   (broadcast RAWTX with sendrawtransaction)
```

The transaction has a single output — the **possession outpoint** — whose locking script is the
note-anchoring carrier `<root> OP_DROP OP_DUP OP_HASH160 <pkh> OP_EQUALVERIFY OP_CHECKSIG`: the
32-byte root rides as pushdata (carried, not executed) ahead of a native P2PKH spend tail, so the
output stays a genuinely spendable UTXO and the act of spending the funded input is bound to
committing the root in one transaction. This satisfies REQ-CHAIN-0003 (state root committed by the
possession transaction), REQ-CHAIN-0001/0002 (standard P2PKH), and the absolute OP_RETURN prohibition
(REQ-CHAIN-0051 / REQ-BUILD-0010). It mirrors `bsvscript::BuildEnvelopeDrop` and
`SPV/03-identity/anchor` (SYS-ENC-001/002, ID-CON-001).

## Verified live

See [ANCHOR_RECEIPT.md](ANCHOR_RECEIPT.md) — a real `idattr` registry root committed on-chain on the
Teranode regtest node, accepted by `sendrawtransaction` and mined into a block.
