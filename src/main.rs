//! Anchor an identity-registry root on-chain (BSV regtest).
//!
//! `keygen`        — generate a secp256k1 key + its regtest P2PKH address (to mine coins to).
//! `build-anchor`  — spend a funded P2PKH output and commit the 32-byte anchor root in the tx,
//!                   producing a fully-signed standard BSV transaction (BIP143/FORKID) as raw hex
//!                   for `sendrawtransaction`.
//!
//! The registry anchor root is committed through the SCARCITY / TEA-BSV note-anchoring template,
//! NOT an OP_RETURN data carrier: the root rides as pushdata inside a SPENDABLE locking script
//! (`<root> OP_DROP <P2PKH>`), so the commitment output is itself a standard P2PKH possession
//! outpoint. This satisfies REQ-CHAIN-0003 (the state root is committed by the possession
//! transaction, binding spend + commit in one tx), REQ-CHAIN-0001/0002 (standard P2PKH spend tail),
//! and the absolute OP_RETURN prohibition (REQ-CHAIN-0051 / REQ-BUILD-0010). The carrier mirrors
//! `bsvscript::BuildEnvelopeDrop` / `SPV/03-identity/anchor` (SYS-ENC-001/002, ID-CON-001).

#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use bsv::{
    bytes_to_hex, hash160, p2pkh, push_data, sighash, OutPoint, Transaction, TxIn, TxOut, Txid,
    SIGHASH_ALL, SIGHASH_FORKID,
};

/// `OP_DROP` (0x75): pops and discards the top stack item, leaving the trailing P2PKH to authorise
/// the spend. Not exported by the shared `bsv::script::op` subset, so it is named here.
const OP_DROP: u8 = 0x75;

/// Build the note-anchoring locking script: `<root> OP_DROP OP_DUP OP_HASH160 <pkh> OP_EQUALVERIFY
/// OP_CHECKSIG`. The 32-byte root is pushed then dropped (carried, not executed); the trailing
/// native P2PKH keeps the output a genuinely spendable possession outpoint — never OP_RETURN,
/// never P2SH. Mirrors `bsvscript::BuildEnvelopeDrop` (carrier b).
fn build_anchor_locking_script(root: &[u8], h160: &[u8; 20]) -> Vec<u8> {
    let mut s = Vec::new();
    push_data(&mut s, root);
    s.push(OP_DROP);
    s.extend_from_slice(&p2pkh(h160));
    s
}
use clap::{Parser, Subcommand};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::SecretKey;
use rand::rngs::OsRng;

#[derive(Parser)]
#[command(name = "idattr-onchain")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate a key + regtest P2PKH address.
    Keygen,
    /// Build a signed anchor transaction.
    BuildAnchor {
        #[arg(long)]
        privkey: String,
        #[arg(long)]
        pubkey: String,
        /// Funding outpoint txid (display/big-endian hex, as the node reports it).
        #[arg(long)]
        coinbase_txid: String,
        #[arg(long, default_value_t = 0)]
        vout: u32,
        /// Value of the funded output, in satoshis.
        #[arg(long)]
        value: u64,
        /// The 32-byte anchor root (hex) to commit.
        #[arg(long)]
        anchor_root: String,
        /// Fee in satoshis.
        #[arg(long, default_value_t = 500)]
        fee: u64,
    },
}

/// Regtest/testnet P2PKH version byte for base58check addresses.
const REGTEST_P2PKH_VERSION: u8 = 0x6f;

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Keygen => keygen(),
        Cmd::BuildAnchor {
            privkey,
            pubkey,
            coinbase_txid,
            vout,
            value,
            anchor_root,
            fee,
        } => build_anchor(&privkey, &pubkey, &coinbase_txid, vout, value, &anchor_root, fee),
    }
}

fn keygen() -> Result<()> {
    let sk = SecretKey::random(&mut OsRng);
    let priv_bytes = sk.to_bytes();
    let pub_point = sk.public_key().to_encoded_point(true);
    let pub_bytes = pub_point.as_bytes();
    let h160 = hash160(pub_bytes);
    let address = bs58::encode(h160)
        .with_check_version(REGTEST_P2PKH_VERSION)
        .into_string();

    println!("PRIVKEY {}", hex::encode(priv_bytes));
    println!("PUBKEY {}", hex::encode(pub_bytes));
    println!("H160 {}", hex::encode(h160));
    println!("ADDRESS {address}");
    Ok(())
}

fn build_anchor(
    privkey_hex: &str,
    pubkey_hex: &str,
    coinbase_txid: &str,
    vout: u32,
    value: u64,
    anchor_root_hex: &str,
    fee: u64,
) -> Result<()> {
    let privkey: [u8; 32] = hex::decode(privkey_hex)
        .context("privkey hex")?
        .try_into()
        .map_err(|_| anyhow!("privkey must be 32 bytes"))?;
    let pubkey = hex::decode(pubkey_hex).context("pubkey hex")?;
    let anchor_root = hex::decode(anchor_root_hex).context("anchor_root hex")?;
    anyhow::ensure!(anchor_root.len() == 32, "anchor_root must be 32 bytes");
    anyhow::ensure!(value > fee, "value must exceed fee");

    let h160 = hash160(&pubkey);
    let script_code = p2pkh(&h160); // the funded coinbase output is P2PKH(our key)

    let input = TxIn {
        outpoint: OutPoint {
            txid: Txid::from_display_hex(coinbase_txid).context("coinbase txid")?,
            vout,
        },
        unlocking_script: Vec::new(),
        sequence: 0xffff_ffff,
    };

    // Single output: the possession outpoint. It carries the anchor root via the note-anchoring
    // template (`<root> OP_DROP <P2PKH>`) AND remains a spendable P2PKH UTXO holding the value, so
    // spending the funded input and committing the root are bound in one transaction (REQ-CHAIN-0003).
    let anchor_out = TxOut {
        value: value - fee,
        locking_script: build_anchor_locking_script(&anchor_root, &h160),
    };
    let mut tx = Transaction {
        version: 1,
        inputs: vec![input],
        outputs: vec![anchor_out],
        locktime: 0,
    };

    // Sign input 0 with standard BSV BIP143/FORKID sighash.
    let flags = SIGHASH_ALL | SIGHASH_FORKID;
    let sh = sighash(&tx, 0, &script_code, value, flags).map_err(|e| anyhow!("sighash: {e}"))?;
    let mut sig = ckd::sign_prehash_der(&privkey, sh.internal()).map_err(|e| anyhow!("sign: {e}"))?;
    sig.push(flags);

    let mut unlock = Vec::new();
    push_data(&mut unlock, &sig);
    push_data(&mut unlock, &pubkey);
    tx.inputs[0].unlocking_script = unlock;

    let raw = tx.serialize().map_err(|e| anyhow!("serialize: {e}"))?;
    let txid = tx.txid().map_err(|e| anyhow!("txid: {e}"))?;
    println!("TXID {}", txid.to_display_hex());
    println!("RAWTX {}", bytes_to_hex(&raw));
    Ok(())
}
