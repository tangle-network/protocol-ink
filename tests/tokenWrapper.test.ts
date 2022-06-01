import { expect } from "chai";
import { artifacts, network, patract } from "redspot";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;


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

     function tokenWrapperContractInitParams(sender: any, BobSigner: any, CharlieSigner: any) {
        let tokenName = 'Webb';
        let tokenSymbol = 'Webb';
        let decimal = 4;
        let contractGovernor = sender.address;
        let feeRecipient = CharlieSigner.address;
        let feePercentage = 1;
        let isNativeAllowed = true;
        let wrappingLimit = 10;
        let contractProposalNonce = 0;
        let tokenAddress = BobSigner.address
        let totalSupply = 1_000_000;
        let governorBalance = 1000;

        return {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
        contractProposalNonce, tokenAddress, totalSupply, governorBalance}
    }

    it.skip('Add token address', async () => {
        const { sender, BobSigner, CharlieSigner } = await setup();

        const {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
            contractProposalNonce, tokenAddress, totalSupply, governorBalance} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit, contractProposalNonce, tokenAddress,
            totalSupply, governorBalance);

        //console.log(`token wrapper contract initialized ${tokenWrapperContract}`);

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
    });

    it.skip('Remove token address', async () => {
        const { sender, BobSigner, CharlieSigner } = await setup();
        const {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
            contractProposalNonce, tokenAddress, totalSupply, governorBalance} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)


        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit, contractProposalNonce, tokenAddress,
            totalSupply, governorBalance);

        // first add a token address
        let addTokenFunction = await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)
        expect(addTokenFunction).to.be.ok;

        // validate that address has been added successfully
        let  isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // validate that proposalNonce has increased
        let  newProposalNonce = await tokenWrapperContract.query.nonce();
        expect(newProposalNonce.output).to.be.equal(contractProposalNonce + 1);

        // increase nonce
        // @ts-ignore
        let proposalNonce = Number(newProposalNonce.output) + 1;

        console.log(`proposalNonce is ${proposalNonce}`);

        // now remove token address
        let removeTokenFunction = await tokenWrapperContract.tx.removeTokenAddress(BobSigner.address, proposalNonce);
        expect(removeTokenFunction).to.be.ok;

        // validate that address has been removed successfully
        let isValidAddressAgain = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddressAgain.output).to.equal(false);
        console.log(`new proposalNonce is ${isValidAddressAgain.output}`);

        // validate that proposalNonce has increased
        let  newProposalNonceAgain  = await tokenWrapperContract.query.nonce();
        console.log(`new proposalNonce is ${newProposalNonceAgain.output}`);
        expect(newProposalNonceAgain.output).to.be.equal(proposalNonce);
    });

    it.skip('Update config', async () => {
        const { sender, BobSigner, CharlieSigner } = await setup();
        const {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient,
            feePercentage, isNativeAllowed, wrappingLimit,
            contractProposalNonce, tokenAddress, totalSupply, governorBalance} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage,
            isNativeAllowed, wrappingLimit, contractProposalNonce, tokenAddress,
            totalSupply, governorBalance);


        let newGovernor = BobSigner.address;
        let newIsNativeAllowed = false;
        let newWrappingLimit = 1000;
        let newFeePercentage = 2;
        let newFeeRecipient = sender.address;

        // update config with new states
        let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(newGovernor, newIsNativeAllowed, newWrappingLimit,
            newFeePercentage, newFeeRecipient)
        expect(updateConfigFunction).to.be.ok;

        // validate that new governor is not same as old governor
        let newGovernorFromStorage = await tokenWrapperContract.query.governor();
        expect(newGovernorFromStorage.output).to.equal(newGovernor);
        expect(newGovernorFromStorage.output).to.not.equal(contractGovernor);

        // validate that isNativeAllowed has changed
        let  newIsNativeAllowedFromStorage = await tokenWrapperContract.query.isNativeAllowed();
        expect(newIsNativeAllowedFromStorage.output).to.be.equal(newIsNativeAllowed);
        expect(newIsNativeAllowedFromStorage.output).to.not.equal(isNativeAllowed);

        // validate that isWrappingLimit has changed
        let newWrappingLimitFromStorage = await tokenWrapperContract.query.wrappingLimit();
        expect(Number(newWrappingLimitFromStorage.output)).to.equal(newWrappingLimit);
        expect(Number(newWrappingLimitFromStorage.output)).to.not.equal(wrappingLimit);

        // validate that feePercentage has changed
        let newFeePercentageFromStorage = await tokenWrapperContract.query.feePercentage();
        expect(Number(newFeePercentageFromStorage.output)).to.equal(newFeePercentage);
        expect(Number(newFeePercentageFromStorage.output)).to.not.equal(feePercentage);

        // validate that feeRecipient has changed
        let newFeeRecipientFromStorage = await tokenWrapperContract.query.feeRecipient();
        expect(newFeeRecipientFromStorage.output).to.equal(newFeeRecipient);
        expect(newFeeRecipientFromStorage.output).to.not.equal(feeRecipient);

    });

    it('Test native wrapping functionality', async () => {
        const { sender, BobSigner, CharlieSigner } = await setup();
        const {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
            contractProposalNonce, tokenAddress, totalSupply, governorBalance} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit, contractProposalNonce, tokenAddress,
            totalSupply, governorBalance);

        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(sender.address);
        let initialContractBalance = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(initialSenderWrappedBalance.output)).to.not.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);

        let wrapFunction = await tokenWrapperContract.tx.wrap( null, 0, { value: 1000 });

         expect(wrapFunction).to.be.ok;

        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(sender.address);
        let contractBalanceAfter = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.be.greaterThan(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));


    });

    it.skip('Test unwrap functionality', async () => {
        const { sender, BobSigner, CharlieSigner } = await setup();
        const {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
            contractProposalNonce, tokenAddress, totalSupply, governorBalance} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)


        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory('governed_token_wrapper', sender.address);
        const tokenWrapperContract = await tokenWrapperContractFactory.deploy('new',
            tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit, contractProposalNonce, tokenAddress,
            totalSupply, governorBalance);

        const wrapResponse = await tokenWrapperContract.tx.wrap(sender.address, 1000, { value: 10 });
        console.log(wrapResponse);

        const unwrapResponse = await tokenWrapperContract.tx.unwrap(sender.address, 1000, { value: 10 });
        console.log(unwrapResponse);
    });
})
