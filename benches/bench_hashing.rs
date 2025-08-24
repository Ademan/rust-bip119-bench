use bitcoin::{Amount, CompressedPublicKey, NetworkKind, PrivateKey, PublicKey, ScriptBuf, TapNodeHash};
use bitcoin::secp256k1::{Keypair, Secp256k1, SecretKey};
use bitcoin::secp256k1::rand;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bip119_bench::{
    ctv_from_components,
    ctv_from_transaction,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    let secp = Secp256k1::new();

    let (_ska, pka) = secp.generate_keypair(&mut rand::thread_rng());
    let (_skb, pkb) = secp.generate_keypair(&mut rand::thread_rng());

    let (pka, _) = pka.x_only_public_key();
    let (pkb, _) = pkb.x_only_public_key();

    let a_value = Amount::from_sat(42_000);
    let b_value = Amount::from_sat(999_999);

    let input_index = 1;

    let mut group = c.benchmark_group("CTV Template Calculation 2-in-3-out");
    group.bench_function("from_transaction", |b| b.iter(|| {
        let ctv_hash = ctv_from_transaction(
            black_box(a_value), black_box(pka),
            black_box(b_value), black_box(pkb),
            black_box(input_index));

        black_box(ctv_hash)
    }));
    group.bench_function("from_components", |b| b.iter(|| {
        let ctv_hash = ctv_from_components(
            black_box(a_value), black_box(pka),
            black_box(b_value), black_box(pkb),
            black_box(input_index));

        black_box(ctv_hash)
        
    }));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
