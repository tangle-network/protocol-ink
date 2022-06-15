use ark_bn254::Bn254;
use ark_crypto_primitives::SNARK;
use ark_ec::PairingEngine;
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use ark_std::test_rng;
use arkworks_setups::r1cs::anchor::AnchorR1CSProver;
use arkworks_setups::r1cs::mixer::MixerR1CSProver;
use arkworks_setups::r1cs::vanchor::VAnchorR1CSProver;
use arkworks_utils::Curve;
use std::env::current_dir;
use std::fs::write;

fn save_keys<E: PairingEngine>(proving_key: ProvingKey<E>, verifying_key: VerifyingKey<E>, path: &str) {
	let mut pk = Vec::new();
	let mut vk = Vec::new();
	proving_key.serialize(&mut pk).unwrap();
	verifying_key.serialize(&mut vk).unwrap();

	let mut pk_uncompressed = Vec::new();
	let mut vk_uncompressed = Vec::new();
	proving_key.serialize_uncompressed(&mut pk_uncompressed).unwrap();
	verifying_key.serialize_uncompressed(&mut vk_uncompressed).unwrap();

	let current_path = current_dir().unwrap();
	write(format!("{}/{}/proving_key.bin", current_path.display(), path), pk).unwrap();
	write(format!("{}/{}/verifying_key.bin", current_path.display(), path), vk).unwrap();
	write(format!("{}/{}/proving_key_uncompressed.bin", current_path.display(), path), pk_uncompressed).unwrap();
	write(format!("{}/{}/verifying_key_uncompressed.bin", current_path.display(), path), vk_uncompressed).unwrap();
}

fn generate_mixer_keys<E: PairingEngine, const HEIGHT: usize>(curve: Curve, relative_path: &str) {
    let rng = &mut test_rng();

    // Setup random circuit
    let (c, ..) =
        MixerR1CSProver::<E, HEIGHT>::setup_random_circuit(curve, [0u8; 32], rng).unwrap();
    // Generate the keys
    let (proving_key, verifying_key) = Groth16::<E>::circuit_specific_setup(c, rng).unwrap();

    save_keys(proving_key, verifying_key, relative_path);
}

fn generate_anchor_keys<E: PairingEngine, const HEIGHT: usize, const ANCHOR_CT: usize>(
    curve: Curve,
    relative_path: &str,
) {
    let rng = &mut test_rng();

    // Setup random circuit
    let (c, ..) =
        AnchorR1CSProver::<E, HEIGHT, ANCHOR_CT>::setup_random_circuit(curve, [0u8; 32], rng)
            .unwrap();
    // Generate the keys
    let (proving_key, verifying_key) = Groth16::<E>::circuit_specific_setup(c, rng).unwrap();

    save_keys(proving_key, verifying_key, relative_path);
}

fn generate_vanchor_keys<
    E: PairingEngine,
    const HEIGHT: usize,
    const ANCHOR_CT: usize,
    const INS: usize,
    const OUTS: usize,
>(
    curve: Curve,
    relative_path: &str,
) where
    E::Fr: From<i128>,
{
    let rng = &mut test_rng();

    // Setup random circuit
    let c = VAnchorR1CSProver::<E, HEIGHT, ANCHOR_CT, INS, OUTS>::setup_random_circuit(
        curve, [0u8; 32], rng,
    )
    .unwrap();
    // Generate the keys
    let (proving_key, verifying_key) = Groth16::<E>::circuit_specific_setup(c, rng).unwrap();

    save_keys(proving_key, verifying_key, relative_path);
}
fn main() {
    // Generate Mixer keys with tree of heigth 30
    generate_mixer_keys::<Bn254, 30>(Curve::Bn254, "../mixer/bn254/x5");

    // Generate anchor keys with tree of heigth 30
    // and anchor count of 2
    generate_anchor_keys::<Bn254, 30, 2>(Curve::Bn254, "../fixed-anchor/bn254/x5/2");

	// Generate anchor keys with tree of heigth 30
    // and anchor count of 16
    generate_anchor_keys::<Bn254, 30, 16>(Curve::Bn254, "../fixed-anchor/bn254/x5/16");

	// Generate anchor keys with tree of heigth 30
    // and anchor count of 32
    generate_anchor_keys::<Bn254, 30, 32>(Curve::Bn254, "../fixed-anchor/bn254/x5/32");

    // Generate vanchor keys with tree of height 30
    // and anchor count of 2
    // and number of inputs of 2
    // and number of outputs of 2
    generate_vanchor_keys::<Bn254, 30, 2, 2, 2>(Curve::Bn254, "../vanchor/bn254/x5/2-2-2");

    // Generate vanchor keys with tree of height 30
    // and anchor count of 2
    // and number of inputs of 16
    // and number of outputs of 2
    generate_vanchor_keys::<Bn254, 30, 2, 16, 2>(Curve::Bn254, "../vanchor/bn254/x5/2-16-2");

	// Generate vanchor keys with tree of height 30
    // and anchor count of 32
    // and number of inputs of 2
    // and number of outputs of 2
    generate_vanchor_keys::<Bn254, 30, 32, 2, 2>(Curve::Bn254, "../vanchor/bn254/x5/32-2-2");

    // Generate vanchor keys with tree of height 30
    // and anchor count of 32
    // and number of inputs of 16
    // and number of outputs of 2
    generate_vanchor_keys::<Bn254, 30, 32, 16, 2>(Curve::Bn254, "../vanchor/bn254/x5/32-16-2");
}
