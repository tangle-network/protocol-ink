import { assert, expect } from "chai";
import { artifacts, network, patract } from "redspot";

import {
    generate_proof_js,
    JsNote,
    JsNoteBuilder,
    ProofInputBuilder,
} from '@webb-tools/wasm-utils/njs';

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;


// to call a "method", you use contract.tx.methodName(args). to get a value, you use contract.query.methodName(args).
describe('token-wrapper', () => {
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

    it('Test wrap functionality', async () => {
        const { sender } = await setup();

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
        'Webb', 'Webb', 4, sender.address, sender.address, 10, true, 10, 101, sender.address);

        console.log(`token wrapper contract initialized ${tokenWrapperContract}`);

        let governor = await tokenWrapperContract.query.governor();
        let name = await tokenWrapperContract.query.name();

        console.log(governor);
        console.log(name);

        // wrap
        const wrapResponse = await tokenWrapperContract.tx.wrap(sender.address, 1000, { value: 10 });
        console.log(wrapResponse);
    });

    it('Test unwrap functionality', async () => {
        const { sender } = await setup();

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            'Webb', 'Webb', 4, sender.address, sender.address, 10, true, 10, 101, sender.address);

        console.log(`token wrapper contract initialized ${tokenWrapperContract}`);

        const wrapResponse = await tokenWrapperContract.tx.wrap(sender.address, 1000, { value: 10 });
        console.log(wrapResponse);

        const unwrapResponse = await tokenWrapperContract.tx.unwrap(sender.address, 1000, { value: 10 });
        console.log(unwrapResponse);
    });
})
