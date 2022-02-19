import { expect } from "chai";
import { artifacts, network, patract } from "redspot";


const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;


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
        const poseidonABI = artifacts.readArtifact('poseidon');

        // Mixer verifier instantiation
        const mixerVerifierContractFactory = await getContractFactory('mixer_verifier', sender.address);
        const mixerVerifierContract = await mixerVerifierContractFactory.deploy('new');
        const mixerVerifierABI = artifacts.readArtifact('mixer_verifier');

        // Mixer instantiation
        const levels = 30;
        const depositSize = 100;
        const mixerContractFactory = await getContractFactory('mixer', sender.address);
        const mixerContract = await mixerContractFactory.deploy('new',
            levels,
            depositSize,
            poseidonContract.address,
            mixerVerifierContract.address
        );
        const mixerABI = artifacts.readArtifact('mixer');
    });
})