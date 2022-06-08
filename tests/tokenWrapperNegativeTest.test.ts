import { expect } from "chai";
import { artifacts, network, patract } from "redspot";
import BN from "bn.js";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

export function normalizeBalance(balance: number): BN {
    return new BN(balance).mul(new BN(10).pow(new BN(api.registry.chainDecimals[0])));
}

describe('token-wrapper', () => {
    let tokenWrapperContract: any;
    let tokenWrapperContractFactory: any;
    let sender: any;
    let BobSigner: any;
    let CharlieSigner: any;
    let FerdieSigner: any;
    let EveSigner: any;
    let DaveSigner: any;
    after(() => {
        return api.disconnect()
    });

    before (async () => {
        await api.isReady;
    })

    beforeEach(async () => {
        ({ tokenWrapperContract, tokenWrapperContractFactory, sender, BobSigner, CharlieSigner, FerdieSigner, EveSigner, DaveSigner } =  await setup());
    });

    async function setup() {
        const signerAddresses = await getAddresses();

        const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]))
        const signers = await getSigners()

        const Alice = signerAddresses[0];
        const Bob = signerAddresses[1];
        const Charlie = signerAddresses[2];
        const Ferdie = signerAddresses[3];
        const Eve = signerAddresses[4];
        const Dave = signerAddresses[5];

        const sender = await getRandomSigner(Alice,one.muln(10));
        const CharlieSigner = await getRandomSigner(Charlie,  one.muln(10));
        const BobSigner = await getRandomSigner(Charlie, one.muln(10));
        const FerdieSigner = await getRandomSigner(Ferdie,  one.muln(10));
        const EveSigner = await getRandomSigner(Eve,  one.muln(10));
        const DaveSigner = await getRandomSigner(Dave, one.muln(10));

        const {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
            contractProposalNonce, tokenAddress, totalSupply, governorBalance} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)


        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);

        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit, contractProposalNonce, tokenAddress,
            totalSupply, governorBalance);

        return { sender, Alice, Charlie, CharlieSigner, Bob, BobSigner, Ferdie, FerdieSigner, Eve, EveSigner, Dave, DaveSigner,
            tokenWrapperContractFactory, tokenWrapperContract};
    }

    function tokenWrapperContractInitParams(sender: any, BobSigner: any, CharlieSigner: any) {
        let tokenName = 'Webb';
        let tokenSymbol = 'Webb';
        let decimal = 4;
        let contractGovernor = sender.address;
        let feeRecipient = sender.address;
        let feePercentage = 1;
        let isNativeAllowed = true;
        let wrappingLimit = 10;
        let contractProposalNonce = 2;
        let tokenAddress = BobSigner.address
        let totalSupply = 1_000_000_000_000_000;
        let governorBalance = 9_000_000;


        return {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
            contractProposalNonce, tokenAddress, totalSupply, governorBalance}
    }

    it.skip('Add token address with smaller nonce than what exists', async () => {
        const {tokenName, contractProposalNonce} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

        let governor = await tokenWrapperContract.query.governor();
        let name = await tokenWrapperContract.query.name();

        console.log(`governor is ${governor.output}`);
        console.log(`token name is ${name.output}`);

        expect(governor.output).to.equal(sender.address);
        expect(name.output).to.equal(tokenName);

        let signer = tokenWrapperContract.signer;

        console.log(`signer is ${signer}`);

        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let  isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // validate that proposalNonce has increased
        let  newProposalNonce = await tokenWrapperContract.query.nonce();
        expect(newProposalNonce.output).to.be.equal(contractProposalNonce + 1);

        expect(await tokenWrapperContract.tx.killContract({ value: 10 }))
    });
})
