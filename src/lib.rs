use bitcoin::hashes::{Hash, sha256};
use bitcoin::opcodes::all::OP_PUSHBYTES_32;
use bitcoin::secp256k1::XOnlyPublicKey;
use bitcoin::{absolute, Amount, blockdata::transaction, consensus::Encodable, key::TweakedPublicKey, io::Write, Opcode, OutPoint, ScriptBuf, Sequence, Transaction, Txid, TxIn, TxOut, WitnessVersion, Witness};
use bip119::{DefaultCheckTemplateVerifyHash, hash_sequences};

const PAY_TO_ANCHOR_SCRIPT_BYTES: &[u8] = &[0x51, 0x02, 0x4e, 0x73];

pub fn ctv_from_transaction(
    a_value: Amount, a: XOnlyPublicKey,
    b_value: Amount, b: XOnlyPublicKey,
    input_index: u32,
) -> DefaultCheckTemplateVerifyHash {
    let dummy_prevout: OutPoint = OutPoint {
        txid: Txid::from_byte_array([0u8; 32]),
        vout: 0,
    };

    let transaction = Transaction {
        version: transaction::Version::non_standard(3),
        lock_time: absolute::LockTime::ZERO,
        input: vec![
            TxIn {
                previous_output: dummy_prevout,
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: Witness::new(),
            },
            TxIn {
                previous_output: dummy_prevout,
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: Witness::new(),
            },
        ],
        output: vec![
            TxOut {
                value: a_value,
                script_pubkey: ScriptBuf::new_p2tr_tweaked(TweakedPublicKey::dangerous_assume_tweaked(a)),
            },
            TxOut {
                value: b_value,
                script_pubkey: ScriptBuf::new_p2tr_tweaked(TweakedPublicKey::dangerous_assume_tweaked(b)),
            },
            TxOut {
                value: Amount::ZERO,
                script_pubkey: ScriptBuf::from_bytes(PAY_TO_ANCHOR_SCRIPT_BYTES.to_vec()),
            },
        ],
    };

    DefaultCheckTemplateVerifyHash::from_transaction(&transaction, input_index)
}

pub fn ctv_from_components(
    a_value: Amount, a: XOnlyPublicKey,
    b_value: Amount, b: XOnlyPublicKey,
    input_index: u32,
) -> DefaultCheckTemplateVerifyHash {
    let sequences = [Sequence::ZERO, Sequence::ZERO];
    let sequences_sha256 = hash_sequences(sequences.iter().cloned());

    let outputs_sha256 = {
        let mut sha256 = sha256::Hash::engine();

        let taproot_script_pubkey_len = 1 + 1 + 32;
        let segwit_v1_opcode = Opcode::from(WitnessVersion::V1).to_u8();
        let taproot_script_prefix = [
            taproot_script_pubkey_len,
            segwit_v1_opcode,
            OP_PUSHBYTES_32.to_u8(),
        ];

        // Write output 0
        a_value.consensus_encode(&mut sha256).unwrap();
        sha256.write(&taproot_script_prefix).unwrap();
        sha256.write(&a.serialize()).unwrap();

        // Write output 1
        b_value.consensus_encode(&mut sha256).unwrap();
        sha256.write(&taproot_script_prefix).unwrap();
        sha256.write(&b.serialize()).unwrap();

        // Write output 2
        Amount::ZERO.consensus_encode(&mut sha256).unwrap();
        sha256.write(&[PAY_TO_ANCHOR_SCRIPT_BYTES.len() as u8]).unwrap();
        sha256.write(&PAY_TO_ANCHOR_SCRIPT_BYTES).unwrap();

        sha256::Hash::from_engine(sha256)
    };

    DefaultCheckTemplateVerifyHash::from_components(
        transaction::Version::non_standard(3),
        absolute::LockTime::ZERO,
        sequences.len() as u32, // input count
        None, // No script sigs
        sequences_sha256,
        3, // output count
        outputs_sha256,
        input_index, // First input
    )
}

#[cfg(test)]
mod test {
    use super::*;

    use bitcoin::secp256k1::{rand, Secp256k1};

    #[test]
    fn test_equivalence() {
        let secp = Secp256k1::new();

        let (_ska, pka) = secp.generate_keypair(&mut rand::thread_rng());
        let (_skb, pkb) = secp.generate_keypair(&mut rand::thread_rng());

        let (pka, _) = pka.x_only_public_key();
        let (pkb, _) = pkb.x_only_public_key();

        let a_value = Amount::from_sat(42_000);
        let b_value = Amount::from_sat(999_999);


        for input_index in [0, 1] {
            assert_eq!(
                ctv_from_transaction(
                    a_value, pka,
                    b_value, pkb,
                    input_index
                ),
                ctv_from_components(
                    a_value, pka,
                    b_value, pkb,
                    input_index
                ),
            );
        }
    }
}
