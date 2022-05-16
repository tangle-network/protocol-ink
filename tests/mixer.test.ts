import { assert, expect } from "chai";
import { artifacts, network, patract } from "redspot";

import {
  Note,
  NoteGenInput,
  ProvingManagerSetupInput,
  ProvingManagerWrapper,
} from '@webb-tools/sdk-core';

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

async function generateDeposit(amount: number): Promise<Note> {
  const noteInput: NoteGenInput = {
    protocol: 'mixer',
    version: 'v2',
    sourceChain: '5',
    targetChain: '5',
    sourceIdentifyingData: '3',
    targetIdentifyingData: '3',
    tokenSymbol: 'WEBB',
    amount: '1',
    denomination: '18',
    backend: 'Arkworks',
    hashFunction: 'Poseidon',
    curve: 'Bn254',
    width: '3',
    exponentiation: '5',
  };
  const note = await Note.generateNote(noteInput);
  return note
}

// to call a "method", you use contract.tx.methodName(args). to get a value, you use contract.query.methodName(args).
describe('mixer', () => {
  after(() => {
    return api.disconnect()
  });

  async function setup() {
    await api.isReady;
    const signerAddresses = await getAddresses();
    const Alice = signerAddresses[0];
    const sender = await getRandomSigner(Alice, '20000 UNIT');
  
    return { sender, Alice };
  }

  it('Creates a new instance of the mixer', async () => {
    const { sender } = await setup();

    // Poseidon instantiation
    const poseidonContractFactory = await getContractFactory('poseidon', sender.address);
    const poseidonContract = await poseidonContractFactory.deploy('new');

    // Mixer verifier instantiation
    const mixerVerifierContractFactory = await getContractFactory('mixer_verifier', sender.address);
    const mixerVerifierContract = await mixerVerifierContractFactory.deploy('new');

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
    console.log(await mixerContract.query.levels());
    console.log(await mixerContract.query.depositSize());

    // Mixer deposit
    let note = await generateDeposit(depositSize);
    let commitment = note.getLeaf()
    await mixerContract.tx.deposit(commitment, { value: depositSize });
  });
})
