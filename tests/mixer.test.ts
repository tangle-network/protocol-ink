import { assert, expect } from "chai";
import { artifacts, network, patract } from "redspot";

import {
  generate_proof_js,
  JsNote,
  JsNoteBuilder,
  ProofInputBuilder,
} from '@webb-tools/wasm-utils/njs';
import BN from "bn.js";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

export function generateDeposit(amount: number) {
  let noteBuilder = new JsNoteBuilder();
  noteBuilder.protocol('mixer');
  noteBuilder.version('v2');

  noteBuilder.sourceChainId('1');
  noteBuilder.targetChainId('1');

  noteBuilder.tokenSymbol('WEBB');
  noteBuilder.amount('1');
  noteBuilder.sourceIdentifyingData('3');
  noteBuilder.targetIdentifyingData('3');
  noteBuilder.denomination('18');

  noteBuilder.backend('Arkworks');
  noteBuilder.hashFunction('Poseidon');
  noteBuilder.curve('Bn254');
  noteBuilder.width('3');
  noteBuilder.exponentiation('5');
  const note = noteBuilder.build();

  return note;
}

// to call a "method", you use contract.tx.methodName(args). to get a value, you use contract.query.methodName(args).
describe('mixer', () => {
  after(() => {
    return api.disconnect()
  });

  async function setup() {
    console.log("trying setup")
    await api.isReady;
    const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
    const signerAddresses = await getAddresses();
    const Alice = signerAddresses[0];
    const sender = await getRandomSigner(Alice, one.muln(10));


    console.log("finished trying set up")
    return { sender, Alice };
  }

  it.only('Creates a new instance of the mixer', async () => {
    const { sender } = await setup();

    console.log("creating poseidon contract");
    // Poseidon instantiation
    const poseidonContractFactory = await getContractFactory('poseidon', sender.address);
    const poseidonContract = await poseidonContractFactory.deploy('new');

    console.log("poseidon deployed");

    // Mixer verifier instantiation
    const mixerVerifierContractFactory = await getContractFactory('mixer_verifier', sender.address);
    const mixerVerifierContract = await mixerVerifierContractFactory.deploy('new');

    console.log("mixer verifier deployed");

   // console.log(poseidonContract.abi.info.source.wasmHash);
   // console.log(mixerVerifierContract.abi.info.source.wasmHash);

    // Mixer instantiation
    const randomVersion = Math.floor(Math.random() * 10000);
    const levels = 30;
    const depositSize = 100;
    const mixerContractFactory = await getContractFactory('mixer', sender.address);
    const mixerContract = await mixerContractFactory.deploy('new',
      levels,
      depositSize,
      randomVersion,
      poseidonContract.abi.info.source.wasmHash,
      mixerVerifierContract.abi.info.source.wasmHash,
    );

    await mixerContract.query.levels();
    await mixerContract.query.depositSize();

    // Mixer deposit
    let note = generateDeposit(depositSize);
    let commitment = note.getLeafCommitment();

    console.log("sending deposit");
    const resp = await mixerContract.tx.deposit(commitment, { value: depositSize });
    console.log(resp);
  });
})
