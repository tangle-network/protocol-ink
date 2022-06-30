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
import {startContractNode} from "./util";

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
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await startContractNode();
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

  it.only("Test deposit and withdraw functionality", async () => {
    const { sender, BobSigner } = await setup();

    console.log("instantiating poseidon");
    // Poseidon instantiation
    const poseidonContractFactory = await getContractFactory(
      "poseidon",
      sender.address
    );
    const poseidonContract = await poseidonContractFactory.deploy("new");

    console.log("instantiating verifier");
    // Mixer verifier instantiation
    const mixerVerifierContractFactory = await getContractFactory(
      "mixer_verifier",
      sender.address
    );
    const mixerVerifierContract = await mixerVerifierContractFactory.deploy(
      "new"
    );

    console.log(poseidonContract.abi.info.source.wasmHash.toHex());
    console.log(mixerVerifierContract.abi.info.source.wasmHash.toHex());

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
      poseidonContract.abi.info.source.wasmHash,
      mixerVerifierContract.abi.info.source.wasmHash
    );

    await mixerContract.query.levels();
    await mixerContract.query.depositSize();

    // Mixer deposit
    let note = generateDeposit(depositSize);
    let commitment = note.getLeafCommitment();

    console.log(`commitment ${u8aToHex(commitment)}`)

    const noteGenInput: NoteGenInput = {
      protocol: "mixer",
      version: "v2",
      sourceChain: "1",
      targetChain: "1",
      amount: "1",
      tokenSymbol: "WEBB",
      sourceIdentifyingData: "3",
      targetIdentifyingData: "3",
      denomination: "18",
      backend: "Arkworks",
      hashFunction: "Poseidon",
      curve: "Bn254",
      width: "3",
      exponentiation: "5",
    };

    // Generate deposit note, and the secrets associated with the deposit.
    //const note = await Note.generateNote(noteGenInput);

    // The leaf is the value inserted on-chain. Users can prove knowledge of
    // the secrets which were used in generating a leaf, without revealing the secrets.
    //const commitment = note.getLeaf();

    const contractId = await mixerContract.query.nativeContractAccountId();
    console.log(`contractId is ${contractId.output}`);

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

    //console.log(`provingInput is ${provingInput}`);

    let proof = await pm.prove("mixer", provingInput);
    let proof_bytes = `0x${proof.proof}` as any;
    let root = `0x${proof.root}`;
    let nullifier_hash = `0x${proof.nullifierHash}`;
    let recipient = BobSigner.address;
    let relayer = BobSigner.address;
    let fee = 999979;
    let refund = 1;

    console.log(`nullifier is ${nullifier_hash}`)

    let contractBalance = await mixerContract.query.nativeContractBalance();

    console.log(`contract balance is ${contractBalance.output}`);
    //const publicInputsFunction = await mixerContract.query.formulatePublicInput(root, nullifier_hash, recipient, relayer, fee, refund);
    //expect(publicInputsFunction).to.be.ok;

    //const publicInputsFunctionKeccak = await mixerContract.query.formulatePublicInputWithKeccak(root, nullifier_hash, recipient, relayer, fee, refund);

    // @ts-ignore
    //const publicInputs = [publicInputsFunction.output.toString('hex').replace('0x', '')];
    // @ts-ignore
    //const publicInputsKeccak = [publicInputsFunctionKeccak.output.toString('hex').replace('0x', '')];
    let pub = "";
    proof.publicInputs.forEach(function (val) {
      pub += val;
    });
    console.log(`proof input is ${proof.publicInputs}`);
    console.log(`proof input mode ${pub}`);
    //    console.log(`public input is ${publicInputsFunction.output}`)
    //  console.log(`public input keccak is ${publicInputsFunctionKeccak.output}`)
    //console.log(`proving key is ${provingKey.toString('hex').replace('0x', '')}`)

    //const isValidProof = verify_js_proof(proof.proof, proof.publicInputs, provingKey.toString('hex').replace('0x', ''), 'Bn254');
    //expect(isValidProof).to.be.true;
    //console.log(`isValidProof is ${isValidProof.valueOf()}`)

    console.log("sending deposit");
    const depositFunction = await mixerContract.tx.deposit(commitment, {
      value: depositSize,
    });
    expect(depositFunction).to.be.ok;

    const sendFundToContract = await mixerContract.tx.sendFundToContract({
      value: depositSize + depositSize,
    });
    expect(sendFundToContract).to.be.ok;

    let contractBalanceBeforeWithdraw =
      await mixerContract.query.nativeContractBalance();

    console.log(`contract balance before withdraw is ${contractBalanceBeforeWithdraw.output}`);

  //  console.log(`is known nullifier payable ${isNullifierKnownPayable.output}`);

    console.log("sending withdrawal");
    const withdrawFunction = await mixerContract.tx.withdraw(
        {proof_bytes,
      root,
      nullifier_hash,
      recipient,
      relayer,
      fee,
      refund}
    );
    expect(withdrawFunction).to.be.ok;

    let contractBalanceAfterWithdraw =
        await mixerContract.query.nativeContractBalance();

    console.log(`contract balance after withdraw is ${contractBalanceAfterWithdraw.output}`);

    // Expect contract balance to be lower after withdrawal
    // @ts-ignore
    expect(Number(contractBalanceAfterWithdraw.output) < Number(contractBalanceBeforeWithdraw.output)).to.be.true;
  });
});
