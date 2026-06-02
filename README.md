# idattr-onchain

Anchor an identity-registry root **on the BSV chain**. Builds and signs a standard BSV transaction
(BIP143 / SIGHASH_FORKID) that spends a funded P2PKH output and commits the 32-byte registry/anchor
root, for broadcast to a Teranode regtest node via `sendrawtransaction`.

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

The transaction has two outputs: `vout0` commits the anchor root (data output), `vout1` is P2PKH
change. NOTE: a strict SCARCITY deployment (REQ-CHAIN-0003) commits the root through the TEA-BSV
note-anchoring template rather than a data carrier; funding/signing/broadcast are identical.

## Verified live

See [ANCHOR_RECEIPT.md](ANCHOR_RECEIPT.md) — a real `idattr` registry root committed on-chain on the
Teranode regtest node, accepted by `sendrawtransaction` and mined into a block.
