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
        let contractProposalNonce = 0;
        let tokenAddress = BobSigner.address
        let totalSupply = 1_000_000_000_000_000;
        let governorBalance = 9_000_000;


        return {tokenName, tokenSymbol, decimal, contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,
        contractProposalNonce, tokenAddress, totalSupply, governorBalance}
    }

    it('Add token address', async () => {
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
        await api.disconnect();
    });

    it('Remove token address', async () => {
        const {contractProposalNonce,} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

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

        expect(await tokenWrapperContract.tx.killContract({ value: 1000 }))
    });

    it('Update config', async () => {
        const {contractGovernor, feeRecipient, feePercentage, isNativeAllowed, wrappingLimit,} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

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
        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(sender.address);
        let initialContractBalance = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(initialSenderWrappedBalance.output)).to.not.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);

        let wrapFunction = await tokenWrapperContract.tx.wrap( null, 10, { value: 1500 });

         expect(wrapFunction).to.be.ok;

         // to validate that psp22 token has been minted for sender
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(sender.address);
        // to validate that native funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.be.greaterThan(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
    });

    it('Test psp22 wrapping functionality', async () => {
        const {feeRecipient, contractProposalNonce} = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner, )

        // first add token address
        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // secondly set psp22 allowance
        let allowedAmount = 500000;
        expect( await tokenWrapperContract.tx.setPsp22Allowance(sender.address, allowedAmount)).to.be.ok;

        let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(sender.address, sender.address);

        // validate that psp22 allowance was set
        expect( Number(allowanceSet.output)).to.be.equal(allowedAmount);

        let newFeeRecipient = EveSigner.address;

        // update config with new states
        let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(0,0,0,1,newFeeRecipient)
        expect(updateConfigFunction).to.be.ok;

        // validate that feeRecipient has changed
        let newFeeRecipientFromStorage = await tokenWrapperContract.query.feeRecipient();
        expect(newFeeRecipientFromStorage.output).to.equal(EveSigner.address);
        expect(newFeeRecipientFromStorage.output).to.not.equal(feeRecipient);


        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(sender.address);
        let initialContractBalance = await tokenWrapperContract.query.psp22ContractBalance();
        let initialFeeRecipientBalance = await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

        expect(Number(initialSenderWrappedBalance.output)).to.not.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);
        expect(Number(initialFeeRecipientBalance.output)).to.equal(0);

        // now do wrapping
        let wrapFunction = await tokenWrapperContract.tx.wrap( BobSigner.address, 10, { value: 10 });
        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for sender
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(sender.address);
        // to validate that psp22 token funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.psp22ContractBalance();
        // to validate that fee has been transferred to the fee recipient
        let feeRecipientBalanceAfter = await tokenWrapperContract.query.psp22Balance(feeRecipient);


        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);
        expect(Number(feeRecipientBalanceAfter.output)).to.not.equal(0);

        //expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
        expect(Number(feeRecipientBalanceAfter.output)).to.be.greaterThan(Number(initialFeeRecipientBalance.output));

        //expect(await tokenWrapperContract.tx.terminateContract(DaveSigner.address)).to.be.ok

    });

    it('Test native wrapping for functionality', async () => {

        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.nativeContractBalance();


        expect(Number(initialSenderWrappedBalance.output)).to.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);

        let wrapFunction = await tokenWrapperContract.tx.wrapFor( null, FerdieSigner.address, 0, { value: 1000 });

        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for sender, in this case Ferdie
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        // to validate that native funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.be.greaterThan(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));


    });

    it('Test psp22 wrapping for functionality', async () => {
        const {feeRecipient, contractProposalNonce, } = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

        // first add token address
        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
        let allowedAmount = 500000;
        expect( await tokenWrapperContract.tx.setPsp22AllowanceForOwner(FerdieSigner.address, sender.address, allowedAmount)).to.be.ok;

        let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(FerdieSigner.address, sender.address);

        // validate that psp22 allowance was set
        expect( Number(allowanceSet.output)).to.be.equal(allowedAmount);

        // update config with new states
        let newFeeRecipient = EveSigner.address;
        let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(0,0,0,1,newFeeRecipient)
        expect(updateConfigFunction).to.be.ok;

        // validate that feeRecipient has changed
        let newFeeRecipientFromStorage = await tokenWrapperContract.query.feeRecipient();
        expect(newFeeRecipientFromStorage.output).to.equal(EveSigner.address);
        expect(newFeeRecipientFromStorage.output).to.not.equal(feeRecipient);

        // Fund Ferdies account
        expect(await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)).to.be.ok;

        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.psp22ContractBalance();
        let initialFeeRecipientBalance = await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

        console.log(`Ferdie balance is ${initialSenderWrappedBalance.output}`);

        expect(Number(initialSenderWrappedBalance.output)).to.not.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);
        expect(Number(initialFeeRecipientBalance.output)).to.equal(0);

        // now do wrapping for Ferdie
        let wrapFunction = await tokenWrapperContract.tx.wrapFor( BobSigner.address, FerdieSigner.address, 10, { value: 10 });
        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for Ferdie
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        // to validate that psp22 token funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.psp22ContractBalance();
        // to validate that fee has been transferred to the fee recipient
        let feeRecipientBalanceAfter = await tokenWrapperContract.query.psp22Balance(feeRecipient);


        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);
        expect(Number(feeRecipientBalanceAfter.output)).to.not.equal(0);

        //expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
        expect(Number(feeRecipientBalanceAfter.output)).to.be.greaterThan(Number(initialFeeRecipientBalance.output));

    });

    it('Test native wrapping for and send to functionality', async () => {

        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.nativeContractBalance();


        expect(Number(initialSenderWrappedBalance.output)).to.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);

        let wrapFunction = await tokenWrapperContract.tx.wrapForAndSendTo( null, FerdieSigner.address, 0, EveSigner.address, { value: 1000 });

        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for the recipient specified, in this case Eve
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        // to validate that native funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.be.greaterThan(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
    });

    it('Test psp22 wrapping for and send to functionality', async () => {
        const {feeRecipient, contractProposalNonce, } = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

        // first add token address
        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
        let allowedAmount = 500000;
        expect( await tokenWrapperContract.tx.setPsp22AllowanceForOwner(FerdieSigner.address, sender.address, allowedAmount)).to.be.ok;

        let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(FerdieSigner.address, sender.address);

        // update config with new states
        let newFeeRecipient = EveSigner.address;
        let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(0,0,0,1,newFeeRecipient)
        expect(updateConfigFunction).to.be.ok;

        // validate that feeRecipient has changed
        let newFeeRecipientFromStorage = await tokenWrapperContract.query.feeRecipient();
        expect(newFeeRecipientFromStorage.output).to.equal(EveSigner.address);
        expect(newFeeRecipientFromStorage.output).to.not.equal(feeRecipient);

        // validate that psp22 allowance was set
        expect( Number(allowanceSet.output)).to.be.equal(allowedAmount);

        // Fund Ferdies account
        expect(await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)).to.be.ok;

        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.psp22ContractBalance();
        let initialFeeRecipientBalance = await tokenWrapperContract.query.psp22Balance(feeRecipient);

        console.log(`EveSigner balance is ${initialSenderWrappedBalance.output}`);

        expect(Number(initialSenderWrappedBalance.output)).to.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);
        //expect(Number(initialFeeRecipientBalance.output)).to.equal(0);

        // now do wrapping for Ferdie
        let wrapFunction = await tokenWrapperContract.tx.wrapForAndSendTo( BobSigner.address, FerdieSigner.address, 10, EveSigner.address, { value: 10 });
        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for Ferdie
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        // to validate that psp22 token funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.psp22ContractBalance();
        // to validate that fee has been transferred to the fee recipient
        let feeRecipientBalanceAfter = await tokenWrapperContract.query.psp22Balance(feeRecipient);

        console.log(`FeeRecipient balance is ${feeRecipientBalanceAfter.output}`);


        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);
        //expect(Number(feeRecipientBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
        //expect(Number(feeRecipientBalanceAfter.output)).to.be.greaterThan(Number(initialFeeRecipientBalance.output));

    });

    it('Test native unwrap functionality', async () => {
        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(sender.address);
        let initialContractBalance = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(initialSenderWrappedBalance.output)).to.not.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);

        // first do wrapping first
        let wrapFunction = await tokenWrapperContract.tx.wrap( null, 0, { value: 1000 });

        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for sender
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(sender.address);
        // to validate that native funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.be.greaterThan(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));

        let unwrapFunction = await tokenWrapperContract.tx.unwrap( null, 1000, { value: 1000 });

        expect(unwrapFunction).to.be.ok;

        let senderBurntBalance = await tokenWrapperContract.query.psp22Balance(sender.address);

        expect(Number(senderBurntBalance.output)).to.not.equal(0);

        // validate that balance has reduced for sender
        expect(Number(senderBurntBalance.output)).to.be.lessThan(Number(senderWrappedBalanceAfter.output));
    });

    it('Test native unwrapping for functionality', async () => {
        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.nativeContractBalance();


        expect(Number(initialSenderWrappedBalance.output)).to.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);

        let wrapFunction = await tokenWrapperContract.tx.wrapFor( null, FerdieSigner.address, 0, { value: 1000 });

        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for sender, in this case Ferdie
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        // to validate that native funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.be.greaterThan(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));

        let unwrapFunction = await tokenWrapperContract.tx.unwrapFor( null, 4, sender.address, { value: 1000 });

        expect(unwrapFunction).to.be.ok;

        // to validate that psp22 token has been burnt for sender, in this case Ferdie
        let senderBurntBalance = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);

        expect(Number(senderBurntBalance.output)).to.not.equal(0);

        // validate that balance has reduced for sender
        //expect(Number(senderBurntBalance.output)).to.be.lessThan(Number(senderWrappedBalanceAfter.output));
    });

    it('Test native wrapping for and send to functionality', async () => {
        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.nativeContractBalance();


        expect(Number(initialSenderWrappedBalance.output)).to.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);

        let wrapFunction = await tokenWrapperContract.tx.wrapForAndSendTo( null, FerdieSigner.address, 0, EveSigner.address, { value: 1000 });

        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for the recipient specified, in this case Eve
        let eveWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        // to validate that native funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.nativeContractBalance();

        expect(Number(eveWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);

        expect(Number(eveWrappedBalanceAfter.output)).to.be.greaterThan(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));

        let senderBalance = await tokenWrapperContract.query.psp22Balance(sender.address);

        let unwrapFunction = await tokenWrapperContract.tx.unwrapAndSendTo( null,  2, sender.address, { value: 1000 });

        expect(unwrapFunction).to.be.ok;

        let senderBurntBalance = await tokenWrapperContract.query.psp22Balance(sender.address);

        expect(Number(senderBurntBalance.output)).to.not.equal(0);

        // validate that balance has reduced for sender
        expect(Number(senderBurntBalance.output)).to.be.lessThan(Number(senderBalance.output));
    });

    it('Test psp22 unwrap functionality', async () => {
        const {feeRecipient, contractProposalNonce, } = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)


        // first add token address
        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // secondly set psp22 allowance
        let allowedAmount = 500000;
        expect( await tokenWrapperContract.tx.setPsp22Allowance(sender.address, allowedAmount)).to.be.ok;

        let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(sender.address, sender.address);

        // validate that psp22 allowance was set
        expect( Number(allowanceSet.output)).to.be.equal(allowedAmount);

        // update config with new states
        let newFeeRecipient = EveSigner.address;
        let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(0,0,0,1,newFeeRecipient)
        expect(updateConfigFunction).to.be.ok;

        // validate that feeRecipient has changed
        let newFeeRecipientFromStorage = await tokenWrapperContract.query.feeRecipient();
        expect(newFeeRecipientFromStorage.output).to.equal(EveSigner.address);


        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(sender.address);
        let initialContractBalance = await tokenWrapperContract.query.psp22ContractBalance();
        let initialFeeRecipientBalance = await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

        expect(Number(initialSenderWrappedBalance.output)).to.not.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);
        expect(Number(initialFeeRecipientBalance.output)).to.equal(0);

        // now do wrapping
        let wrapFunction = await tokenWrapperContract.tx.wrap( BobSigner.address, 10, { value: 10 });
        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for sender
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(sender.address);
        // to validate that psp22 token funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.psp22ContractBalance();
        // to validate that fee has been transferred to the fee recipient
        let feeRecipientBalanceAfter = await tokenWrapperContract.query.psp22Balance(feeRecipient);


        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);
        expect(Number(feeRecipientBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
        expect(Number(feeRecipientBalanceAfter.output)).to.be.greaterThan(Number(initialFeeRecipientBalance.output));

        // secondly set psp22 allowance
        let allowedAmountForContract = 500000;
        expect( await tokenWrapperContract.tx.setPsp22AllowanceForOwner(tokenWrapperContract.address, sender.address, allowedAmount)).to.be.ok;

        let allowanceSetForContract = await tokenWrapperContract.query.getPsp22Allowance(tokenWrapperContract.address, sender.address);

        // validate that psp22 allowance was set
        expect( Number(allowanceSetForContract.output)).to.be.equal(allowedAmountForContract);


        // validate that psp22 allowance was set
        expect( Number(allowanceSetForContract.output)).to.be.equal(allowedAmount);
        // now do unwrapping
        let unwrapFunction = await tokenWrapperContract.tx.unwrap( BobSigner.address, 2, { value: 10 });
        expect(unwrapFunction).to.be.ok;

        // to validate that psp22 token funds was transferred from the contract
        let contractBalanceAfterUnwrap =  await tokenWrapperContract.query.psp22ContractBalance();
        expect(Number(contractBalanceAfterUnwrap.output)).to.not.equal(0);

        expect(Number(contractBalanceAfterUnwrap.output)).to.not.equal(Number(contractBalanceAfter.output));
    });

    it('Test psp22 unwrap for functionality', async () => {
        const {feeRecipient, contractProposalNonce, } = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)


        // first add token address
        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
        let allowedAmount = 500000;
        expect( await tokenWrapperContract.tx.setPsp22AllowanceForOwner(FerdieSigner.address, sender.address, allowedAmount)).to.be.ok;

        let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(FerdieSigner.address, sender.address);

        // validate that psp22 allowance was set
        expect( Number(allowanceSet.output)).to.be.equal(allowedAmount);

        // Fund Ferdies account
        expect(await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)).to.be.ok;

        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.psp22ContractBalance();
        let initialFeeRecipientBalance = await tokenWrapperContract.query.psp22Balance(feeRecipient);

        console.log(`Ferdie balance is ${initialSenderWrappedBalance.output}`);

        expect(Number(initialSenderWrappedBalance.output)).to.not.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);
        expect(Number(initialFeeRecipientBalance.output)).to.equal(0);

        // now do wrapping for Ferdie
        let wrapFunction = await tokenWrapperContract.tx.wrapFor( BobSigner.address, FerdieSigner.address, 10, { value: 10 });
        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for Ferdie
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
        // to validate that psp22 token funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.psp22ContractBalance();
        // to validate that fee has been transferred to the fee recipient
        let feeRecipientBalanceAfter = await tokenWrapperContract.query.psp22Balance(feeRecipient);


        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);
        expect(Number(feeRecipientBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
        expect(Number(feeRecipientBalanceAfter.output)).to.be.greaterThan(Number(initialFeeRecipientBalance.output));

        // secondly set psp22 allowance
        let allowedAmountForContract = 500000;
        expect( await tokenWrapperContract.tx.setPsp22AllowanceForOwner(tokenWrapperContract.address, sender.address, allowedAmount)).to.be.ok;

        let allowanceSetForContract = await tokenWrapperContract.query.getPsp22Allowance(tokenWrapperContract.address, sender.address);

        // validate that psp22 allowance for contract was set
        expect( Number(allowanceSetForContract.output)).to.be.equal(allowedAmountForContract);

        // now do unwrapping
        let unwrapFunction = await tokenWrapperContract.tx.unwrapFor( BobSigner.address,  2, FerdieSigner.address, { value: 10 });
        expect(unwrapFunction).to.be.ok;

        // to validate that psp22 token funds was transferred from the contract
        let contractBalanceAfterUnwrap =  await tokenWrapperContract.query.psp22ContractBalance();
        expect(Number(contractBalanceAfterUnwrap.output)).to.not.equal(0);

        expect(Number(contractBalanceAfterUnwrap.output)).to.not.equal(Number(contractBalanceAfter.output));
    });

    it('Test psp22 unwrapping and send to functionality', async () => {
        const {feeRecipient, contractProposalNonce, } = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner)

        // first add token address
        expect( await tokenWrapperContract.tx.addTokenAddress(BobSigner.address, contractProposalNonce + 1)).to.be.ok;

        // validate that address has been added successfully
        let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(BobSigner.address);
        expect(isValidAddress.output).to.equal(true);

        // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
        let allowedAmount = 500000;
        expect( await tokenWrapperContract.tx.setPsp22AllowanceForOwner(FerdieSigner.address, sender.address, allowedAmount)).to.be.ok;

        let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(FerdieSigner.address, sender.address);

        // validate that psp22 allowance was set
        expect( Number(allowanceSet.output)).to.be.equal(allowedAmount);

        // Fund Ferdies account
        expect(await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)).to.be.ok;

        let initialSenderWrappedBalance = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        let initialContractBalance = await tokenWrapperContract.query.psp22ContractBalance();
        let initialFeeRecipientBalance = await tokenWrapperContract.query.psp22Balance(DaveSigner.address);

        console.log(`EveSigner balance is ${initialSenderWrappedBalance.output}`);

        expect(Number(initialSenderWrappedBalance.output)).to.equal(0);
        expect(Number(initialContractBalance.output)).to.equal(0);
        expect(Number(initialFeeRecipientBalance.output)).to.equal(0);

        // update config with new states
        let newFeeRecipient = DaveSigner.address;
        let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(0,0,0,1,newFeeRecipient)
        expect(updateConfigFunction).to.be.ok;

        // validate that feeRecipient has changed
        let newFeeRecipientFromStorage = await tokenWrapperContract.query.feeRecipient();
        expect(newFeeRecipientFromStorage.output).to.equal(DaveSigner.address);

        // now do wrapping for Ferdie
        let wrapFunction = await tokenWrapperContract.tx.wrapForAndSendTo( BobSigner.address, FerdieSigner.address, 10, EveSigner.address, { value: 10 });
        expect(wrapFunction).to.be.ok;

        // to validate that psp22 token has been minted for Ferdie
        let senderWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(EveSigner.address);
        // to validate that psp22 token funds was transferred to the contract
        let contractBalanceAfter = await tokenWrapperContract.query.psp22ContractBalance();
        // to validate that fee has been transferred to the fee recipient
        let feeRecipientBalanceAfter = await tokenWrapperContract.query.psp22Balance(DaveSigner.address);

        console.log(`FeeRecipient balance is ${feeRecipientBalanceAfter.output}`);


        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(0);
        expect(Number(contractBalanceAfter.output)).to.not.equal(0);
        expect(Number(feeRecipientBalanceAfter.output)).to.not.equal(0);

        expect(Number(senderWrappedBalanceAfter.output)).to.not.equal(Number(initialSenderWrappedBalance.output));
        expect(Number(contractBalanceAfter.output)).to.be.greaterThan(Number(initialContractBalance.output));
        expect(Number(feeRecipientBalanceAfter.output)).to.be.greaterThan(Number(initialFeeRecipientBalance.output));

        // set psp22 allowance for contract
        let allowedAmountForContract = 500000;
        expect( await tokenWrapperContract.tx.setPsp22AllowanceForOwner(tokenWrapperContract.address, sender.address, allowedAmount)).to.be.ok;

        let allowanceSetForContract = await tokenWrapperContract.query.getPsp22Allowance(tokenWrapperContract.address, sender.address);

        // validate that psp22 allowance for contract was set
        expect( Number(allowanceSetForContract.output)).to.be.equal(allowedAmountForContract);

        // now do unwrapping
        // now do wrapping for Ferdie
        let unwrapFunction = await tokenWrapperContract.tx.unwrapAndSendTo( BobSigner.address, 1, FerdieSigner.address, { value: 10 });
        expect(unwrapFunction).to.be.ok;

        // to validate that psp22 token funds was transferred from the contract
        let contractBalanceAfterUnwrap =  await tokenWrapperContract.query.psp22ContractBalance();
        expect(Number(contractBalanceAfterUnwrap.output)).to.not.equal(0);

        expect(Number(contractBalanceAfterUnwrap.output)).to.not.equal(Number(contractBalanceAfter.output));
    });

})
