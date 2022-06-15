import { assert, expect } from "chai";
import { artifacts, network, patract } from "redspot";

import {
  generate_proof_js,
  JsNote,
  JsNoteBuilder,
} from "@webb-tools/wasm-utils/njs";
import BN from "bn.js";
import {ArkworksProvingManager, MerkleTree, Note, NoteGenInput, ProvingManagerSetupInput} from "@webb-tools/sdk-core";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;
import { u8aToHex, hexToU8a } from "@polkadot/util";
import path from "path";
import fs from "fs";
import { decodeAddress } from "@polkadot/util-crypto";
import child from "child_process";

export function generateDeposit(amount: number) {
  let noteBuilder = new JsNoteBuilder();
  noteBuilder.protocol("mixer");
  noteBuilder.version("v2");

  noteBuilder.sourceChainId("1");
  noteBuilder.targetChainId("1");

  noteBuilder.tokenSymbol("WEBB");
  noteBuilder.amount("1");
  noteBuilder.sourceIdentifyingData("3");
  noteBuilder.targetIdentifyingData("3");
  noteBuilder.denomination("18");

  noteBuilder.backend("Arkworks");
  noteBuilder.hashFunction("Poseidon");
  noteBuilder.curve("Bn254");
  noteBuilder.width("3");
  noteBuilder.exponentiation("5");
  const note = noteBuilder.build();

  return note;
}

// to call a "method", you use contract.tx.methodName(args). to get a value, you use contract.query.methodName(args).
describe("mixer", () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    console.log("trying setup");
    await api.isReady;
    const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
    const signerAddresses = await getAddresses();
    const Alice = signerAddresses[0];
    const Bob = signerAddresses[1];
    const BobSigner = await getRandomSigner(Alice, one.muln(10));
    const sender = await getRandomSigner(Alice, one.muln(10));

    console.log("finished trying set up");
    return { sender, Alice, BobSigner };
  }

  it.only("Creates a new instance of the mixer", async () => {
    const { sender, BobSigner } = await setup();

    console.log("creating poseidon contract");
    // Poseidon instantiation
    const poseidonContractFactory = await getContractFactory(
      "poseidon",
      sender.address
    );
    const poseidonContract = await poseidonContractFactory.deploy("new");

    console.log("poseidon deployed");

    // Mixer verifier instantiation
    const mixerVerifierContractFactory = await getContractFactory(
      "mixer_verifier",
      sender.address
    );
    const mixerVerifierContract = await mixerVerifierContractFactory.deploy(
      "new"
    );

    console.log("mixer verifier deployed");

    // console.log(poseidonContract.abi.info.source.wasmHash);
    // console.log(mixerVerifierContract.abi.info.source.wasmHash);

    // Mixer instantiation
    const randomVersion = Math.floor(Math.random() * 10000);
    const levels = 30;
    const depositSize = 100;
    const mixerContractFactory = await getContractFactory(
      "mixer",
      sender.address
    );
    const mixerContract = await mixerContractFactory.deploy(
      "new",
      levels,
      depositSize,
      randomVersion,
      poseidonContract.abi.info.source.wasmHash,
      mixerVerifierContract.abi.info.source.wasmHash
    );

    await mixerContract.query.levels();
    await mixerContract.query.depositSize();

    // Mixer deposit
    let note = generateDeposit(depositSize);
    let commitment = note.getLeafCommitment();

    const noteGenInput: NoteGenInput = {
      protocol: 'mixer',
      version: 'v2',
      sourceChain: '1',
      targetChain: '1',
      amount: '1',
      tokenSymbol: 'WEBB',
      sourceIdentifyingData: '3',
      targetIdentifyingData: '3',
      denomination: '18',
      backend: 'Arkworks',
      hashFunction: 'Poseidon',
      curve: 'Bn254',
      width: '3',
      exponentiation: '5',
    };

    // Generate deposit note, and the secrets associated with the deposit.
    //const note = await Note.generateNote(noteGenInput);

    // The leaf is the value inserted on-chain. Users can prove knowledge of
    // the secrets which were used in generating a leaf, without revealing the secrets.
    //const commitment = note.getLeaf();

    console.log("sending deposit");
    const depositFunction = await mixerContract.tx.deposit(commitment, {
      value: depositSize,
    });
    expect(depositFunction).to.be.ok;

    const merkleTree = new MerkleTree(levels, [u8aToHex(commitment)]);
    const pm = new ArkworksProvingManager(undefined);

    const gitRoot = child
      .execSync("git rev-parse --show-toplevel")
      .toString()
      .trim();
    const provingKeyPath = path.join(
      gitRoot,
      "tests",
      "protocol-substrate-fixtures",
      "mixer",
      "bn254",
      "x5",
      "proving_key_uncompressed.bin"
    );
    const provingKey = fs.readFileSync(provingKeyPath);

    const accountId = sender.address;
    const addressHex = u8aToHex(decodeAddress(accountId));
    const relayerAddressHex = u8aToHex(decodeAddress(accountId));

    const provingInput: ProvingManagerSetupInput<"mixer"> = {
      leafIndex: 0,
      provingKey: hexToU8a(provingKey.toString("hex")),
      note: note.serialize(),
      fee: 0,
      refund: 0,
      leaves: [commitment],
      recipient: addressHex.replace("0x", ""),
      relayer: relayerAddressHex.replace("0x", ""),
    };

    console.log(`provingInput is ${provingInput}`);

    let proof = await pm.prove('mixer', provingInput);
    let proof_bytes = `0x${proof.proof}` as any;
    let root = `0x${proof.root}`;
    let nullifier_hash = `0x${proof.nullifierHash}`;
    let recipient = sender.address;
    let relayer =  sender.address;
    let fee = 0;
    let refund = 0;


    console.log("sending withdrawal");
    const withdrawFunction = await mixerContract.tx.withdraw(proof_bytes, root, nullifier_hash, recipient, relayer, fee, refund);
    expect(withdrawFunction).to.be.ok;


  });
});
