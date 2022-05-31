import { assert, expect } from "chai";
import { artifacts, network, patract } from "redspot";

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
        const Bob = signerAddresses[1];
        const Charlie = signerAddresses[1];
        const sender = await getRandomSigner(Alice, '20000 UNIT');
        const CharlieSigner = await getRandomSigner(Charlie, '20000 UNIT');
        const BobSigner = await getRandomSigner(Charlie, '20000 UNIT');

        return { sender, Alice, Charlie, CharlieSigner, Bob, BobSigner };
    }

    it('Add token address', async () => {
        const { sender, BobSigner, CharlieSigner } = await setup();

        let proposalNonce = 0;
        let totalSupply = 1_000_000;
        let governorBalance = 1000;

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            'Webb', 'Webb', 4, sender.address, sender.address, 10, true, 10, proposalNonce, sender.address,
            totalSupply, governorBalance);

        //console.log(`token wrapper contract initialized ${tokenWrapperContract}`);

        let governor = await tokenWrapperContract.query.governor();
        let name = await tokenWrapperContract.query.name();

        console.log(`governor is ${governor.output}`);
        console.log(`token name is ${name.output}`);

        expect(governor.output).to.equal(sender.address);
        expect(name.output).to.equal('Webb');

        let signer = tokenWrapperContract.signer;

        console.log(`signer is ${signer}`);

        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, proposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let  isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // validate that proposalNonce has increased
        let  newProposalNonce = await tokenWrapperContract.query.nonce();
        expect(newProposalNonce.output).to.be.equal(proposalNonce + 1);
    });

    it('Remove token address', async () => {
        const { sender, BobSigner } = await setup();

        let proposalNonce = 0;
        let totalSupply = 1_000_000;
        let governorBalance = 1000;

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', BobSigner.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            'Webb', 'Webb', 4, sender.address, sender.address, 10, true, 10, proposalNonce, sender.address,
            totalSupply, governorBalance);

        // first add a token address
        let addTokenFunction = await tokenWrapperContract.tx.addTokenAddress(sender.address, proposalNonce + 1)
        expect(addTokenFunction).to.be.ok;

        // validate that address has been added successfully
        let  isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(sender.address);
        expect(isValidAddress.output).to.equal(true);

        // validate that proposalNonce has increased
        let  newProposalNonce = await tokenWrapperContract.query.nonce();
        expect(newProposalNonce.output).to.be.equal(proposalNonce + 1);

        // increase nonce
        // @ts-ignore
        proposalNonce = Number(newProposalNonce.output) + 1;

        console.log(`proposalNonce is ${proposalNonce}`);

        // now remove token address
        let removeTokenFunction = await tokenWrapperContract.tx.removeTokenAddress(BobSigner.address, proposalNonce);
        expect(removeTokenFunction).to.be.ok;

        // validate that address has been removed successfully
        let isValidAddressAgain = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddressAgain.output).to.equal(false);

        // validate that proposalNonce has increased
        let  newProposalNonceAgain  = await tokenWrapperContract.query.nonce();
        expect(newProposalNonceAgain.output).to.be.equal(proposalNonce);
    });

    it.skip('Test native wrapping functionality', async () => {
        const { sender, BobSigner } = await setup();

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
        'Webb', 'Webb', 4, sender.address, sender.address, 10, true, 10, 101, sender.address);

        await expect(tokenWrapperContract.tx.wrap(sender.address, 1000, { value: 10 }))
            .to.emit(tokenWrapperContract, "Wrap");


        //const wrapResponse = await tokenWrapperContract.tx.wrap(sender.address, 1000, { value: 10 });
       // await expect(tokenWrapperContract.tx.wrap(sender.address, 1000, { value: 10 })).to.emit(tokenWrapperContract, "Wrap");
        //console.log(wrapResponse);
    });

    it.skip('Test unwrap functionality', async () => {
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
