import { assert, expect } from "chai";
import { artifacts, network, patract } from "redspot";

import {
  generate_proof_js,
  JsNote,
  JsNoteBuilder,
  verify_js_proof,
} from "@webb-tools/wasm-utils/njs/wasm-utils-njs.js";
import BN from "bn.js";
import {
  ArkworksProvingManager,
  MerkleTree,
  Note,
  NoteGenInput,
  ProvingManagerSetupInput,
} from "@webb-tools/sdk-core";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;
import { u8aToHex, hexToU8a } from "@polkadot/util";
import path from "path";
import fs from "fs";
import { decodeAddress } from "@polkadot/util-crypto";
import child from "child_process";
import exp from "constants";
import { killContractNode, startContractNode } from "./util";

async function fetchSubstrateMixerProvingKey() {
  const IPFSUrl =
    "https://ipfs.io/ipfs/QmfQUgqRXCdUiogiRU8ZdLFZD2vqVb9fHpLkL6DsGHwoLH";
  const ipfsKeyRequest = await fetch(IPFSUrl);
  const circuitKeyArrayBuffer = await ipfsKeyRequest.arrayBuffer();
  const circuitKey = new Uint8Array(circuitKeyArrayBuffer);

  return circuitKey;
}

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

// to call a 'method', you use contract.tx.methodName(args). to get a value, you use contract.query.methodName(args).
describe("mixer", () => {
  let childProcess: any;

  before(async () => {
    childProcess = await startContractNode();
    await api.isReady;
  });

  after(() => {
    killContractNode(childProcess);
    return api.disconnect();
  });

  async function setup() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
    const signerAddresses = await getAddresses();
    const Alice = signerAddresses[0];
    const Bob = signerAddresses[1];
    const BobSigner = await getRandomSigner(Alice, one.muln(10));
    const sender = await getRandomSigner(Alice, one.muln(10));

    return { sender, Alice, BobSigner };
  }

  async function setupForPsp() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
    const signerAddresses = await getAddresses();
    const Alice = signerAddresses[0];
    const Bob = signerAddresses[1];
    const BobSigner = await getRandomSigner(Alice, one.muln(10));
    const sender = await getRandomSigner(Alice, one.muln(10));

    return { sender, Alice, BobSigner };
  }

  it.only("Test deposit and withdraw functionality for native tokens", async () => {
    const { sender, BobSigner } = await setup();

    // Poseidon instantiation
    const poseidonContractFactory = await getContractFactory(
      "poseidon",
      sender.address
    );
    const poseidonContract = await poseidonContractFactory.deploy("new");

    // Mixer verifier instantiation
    const mixerVerifierContractFactory = await getContractFactory(
      "mixer_verifier",
      sender.address
    );
    const mixerVerifierContract = await mixerVerifierContractFactory.deploy(
      "new"
    );

    // Mixer instantiation
    const randomVersion = Math.floor(Math.random() * 10000);
    const levels = 30;
    const depositSize = 100000000;
    const mixerContractFactory = await getContractFactory(
      "mixer",
      sender.address
    );
    const mixerContract = await mixerContractFactory.deploy(
      "new",
      levels,
      depositSize,
      randomVersion,
      null,
      poseidonContract.abi.info.source.wasmHash,
      mixerVerifierContract.abi.info.source.wasmHash
    );

    await mixerContract.query.levels();
    await mixerContract.query.depositSize();

    // Mixer deposit
    let note = generateDeposit(depositSize);
    let commitment = note.getLeafCommitment();

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

    const accountId = BobSigner.address;
    const addressHex = u8aToHex(decodeAddress(accountId));

    const provingInput: ProvingManagerSetupInput<"mixer"> = {
      leafIndex: 0,
      provingKey: hexToU8a(provingKey.toString("hex")),
      note: note.serialize(),
      fee: 999979,
      refund: 1,
      leaves: [commitment],
      recipient: addressHex.replace("0x", ""),
      relayer: addressHex.replace("0x", ""),
    };

    let proof = await pm.prove("mixer", provingInput);
    let proof_bytes = `0x${proof.proof}` as any;
    let root = `0x${proof.root}`;
    let nullifier_hash = `0x${proof.nullifierHash}`;
    let recipient = BobSigner.address;
    let relayer = BobSigner.address;
    let fee = 999979;
    let refund = 1;

    let pub = "";
    proof.publicInputs.forEach(function (val) {
      pub += val;
    });
    const depositFunction = await mixerContract.tx.depositNative(commitment, {
      value: depositSize,
    });
    expect(depositFunction).to.be.ok;

    const sendFundToContract = await mixerContract.tx.sendFundToContract({
      value: depositSize + depositSize,
    });
    expect(sendFundToContract).to.be.ok;

    let contractBalanceBeforeWithdraw =
      await mixerContract.query.nativeContractBalance();

    const withdrawFunction = await mixerContract.tx.withdraw({
      proof_bytes,
      root,
      nullifier_hash,
      recipient,
      relayer,
      fee,
      refund,
    });
    expect(withdrawFunction).to.be.ok;

    let contractBalanceAfterWithdraw =
      await mixerContract.query.nativeContractBalance();

    // Expect contract balance to be lower after withdrawal
    // @ts-ignore
    expect(
      Number(contractBalanceAfterWithdraw.output) <
        Number(contractBalanceBeforeWithdraw.output)
    ).to.be.true;
  });

  it("Test deposit and withdraw functionality for psp22 tokens", async () => {
    const { sender, BobSigner } = await setupForPsp();

    // Poseidon instantiation
    const poseidonContractFactory = await getContractFactory(
      "poseidon",
      sender.address
    );
    const poseidonContract = await poseidonContractFactory.deploy("new");

    // Create a new PSP22 contract instance to get an address
    const psp22ContractFactory = await getContractFactory(
      "psp22_token",
      BobSigner.address
    );
    const psp22Contract = await psp22ContractFactory.deploy(
      "new",
      200000,
      0,
      0,
      1
    );

    // Mixer verifier instantiation
    const mixerVerifierContractFactory = await getContractFactory(
      "mixer_verifier",
      sender.address
    );
    const mixerVerifierContract = await mixerVerifierContractFactory.deploy(
      "new"
    );

    // Mixer instantiation
    const randomVersion = Math.floor(Math.random() * 10000);
    const levels = 30;
    const depositSize = 100_000_000;
    const mixerContractFactory = await getContractFactory(
      "mixer",
      sender.address
    );
    const mixerContract = await mixerContractFactory.deploy(
      "new",
      levels,
      depositSize,
      randomVersion,
      psp22Contract.address,
      poseidonContract.abi.info.source.wasmHash,
      mixerVerifierContract.abi.info.source.wasmHash
    );

    await mixerContract.query.levels();
    await mixerContract.query.depositSize();

    // Mixer deposit
    let note = generateDeposit(depositSize);
    let commitment = note.getLeafCommitment();

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

    const accountId = BobSigner.address;
    const addressHex = u8aToHex(decodeAddress(accountId));

    const provingInput: ProvingManagerSetupInput<"mixer"> = {
      leafIndex: 0,
      provingKey: hexToU8a(provingKey.toString("hex")),
      note: note.serialize(),
      fee: 999979,
      refund: 1,
      leaves: [commitment],
      recipient: addressHex.replace("0x", ""),
      relayer: addressHex.replace("0x", ""),
    };

    let proof = await pm.prove("mixer", provingInput);
    let proof_bytes = `0x${proof.proof}` as any;
    let root = `0x${proof.root}`;
    let nullifier_hash = `0x${proof.nullifierHash}`;
    let recipient = BobSigner.address;
    let relayer = BobSigner.address;
    let fee = 999979;
    let refund = 1;

    let pub = "";
    proof.publicInputs.forEach(function (val) {
      pub += val;
    });

    // insert balance for Alice(sender)
    let insertBalanceFunction = await mixerContract.tx.insertPsp22Balance(
      sender.address,
      10_000_000_000
    );
    expect(insertBalanceFunction).to.be.ok;

    let senderBalance = await mixerContract.query.psp22Balance(sender.address);

    const depositFunction = await mixerContract.tx.depositPsp22(
      commitment,
      depositSize
    );
    expect(depositFunction).to.be.ok;

    const sendFundToContract = await mixerContract.tx.sendFundToContract({
      value: depositSize + depositSize,
    });
    expect(sendFundToContract).to.be.ok;

    let contractBalanceBeforeWithdraw =
      await mixerContract.query.psp22ContractBalance();

    // Set the PSP22 contract allowance
    let allowedAmount = 1000000000;
    const setAllowanceResult = await mixerContract.tx.setPsp22AllowanceForOwner(
      psp22Contract.address,
      sender.address,
      allowedAmount
    );
    expect(setAllowanceResult).to.be.ok;

    let allowanceSet = await mixerContract.query.getPsp22Allowance(
      sender.address,
      mixerContract.address
    );

    // Verify that the PSP22 allowance is set
    expect(Number(allowanceSet.output) === allowedAmount);

    const withdrawFunction = await mixerContract.tx.withdraw({
      proof_bytes,
      root,
      nullifier_hash,
      recipient,
      relayer,
      fee,
      refund,
    });
    expect(withdrawFunction).to.be.ok;

    let contractBalanceAfterWithdraw =
      await mixerContract.query.psp22ContractBalance();
    // Expect contract balance to be lower after withdrawal
    // @ts-ignore
    expect(
      Number(contractBalanceAfterWithdraw.output) <
        Number(contractBalanceBeforeWithdraw.output)
    ).to.be.true;
  });
});
