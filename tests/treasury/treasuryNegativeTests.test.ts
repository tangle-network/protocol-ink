import { expect } from "chai";
import { artifacts, network, patract } from "redspot";
import BN from "bn.js";
import { killContractNode, startContractNode } from "../util";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

export function normalizeBalance(balance: number): BN {
    return new BN(balance).mul(
        new BN(10).pow(new BN(api.registry.chainDecimals[0]))
    );
}

describe("treasury-tests", () => {
    let treasuryContract: any;
    let treasuryContractFactory: any;
    let sender: any;
    let BobSigner: any;
    let CharlieSigner: any;
    let FerdieSigner: any;
    let EveSigner: any;
    let DaveSigner: any;
    let psp22Contract: any;
    let childProcess: any;
    let handler: any;
    after(() => {
        killContractNode(childProcess);
        return api.disconnect();
    });

    before(async () => {
        childProcess = await startContractNode();
        await api.isReady;
    });

    async function getSigner() {
        const signerAddresses = await getAddresses();
        const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
        const signers = await getSigners();

        const Alice = signerAddresses[0];
        const Bob = signerAddresses[1];
        const Charlie = signerAddresses[2];
        const Ferdie = signerAddresses[3];
        const Eve = signerAddresses[4];
        const Dave = signerAddresses[5];

        const sender = await getRandomSigner(signers[0], one.muln(10));
        const CharlieSigner = await getRandomSigner(signers[0], one.muln(10));
        const BobSigner = await getRandomSigner(signers[0], one.muln(10));
        const FerdieSigner = await getRandomSigner(signers[0], one.muln(10));
        const EveSigner = await getRandomSigner(signers[0], one.muln(10));
        const DaveSigner = await getRandomSigner(signers[0], one.muln(10));

        return {
            sender,
            Alice,
            Charlie,
            CharlieSigner,
            Bob,
            BobSigner,
            Ferdie,
            FerdieSigner,
            Eve,
            EveSigner,
            Dave,
            DaveSigner,
        }
    }

    async function setup(handler: any) {
        const signerAddresses = await getAddresses();

        const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
        const signers = await getSigners();

        const Alice = signerAddresses[0];
        const Bob = signerAddresses[1];
        const Charlie = signerAddresses[2];
        const Ferdie = signerAddresses[3];
        const Eve = signerAddresses[4];
        const Dave = signerAddresses[5];

        const sender = await getRandomSigner(signers[0], one.muln(10));
        const CharlieSigner = await getRandomSigner(signers[0], one.muln(10));
        const BobSigner = await getRandomSigner(signers[0], one.muln(10));
        const FerdieSigner = await getRandomSigner(signers[0], one.muln(10));
        const EveSigner = await getRandomSigner(signers[0], one.muln(10));
        const DaveSigner = await getRandomSigner(signers[0], one.muln(10));

        // token wrapper instantiation
        const treasuryContractFactory = await getContractFactory(
            "treasury",
            sender.address
        );

        const treasuryContract = await treasuryContractFactory.deploy(
            "new",
            handler.address
        );

        // create a psp22 token contract to use as token address
        const psp22ContractFactory = await getContractFactory(
            "psp22_token",
            BobSigner.address
        );

        const psp22Contract = await psp22ContractFactory.deploy(
            "new",
            1000000,
            0,
            0,
            1
        );

        return {
            sender,
            Alice,
            Charlie,
            CharlieSigner,
            Bob,
            BobSigner,
            Ferdie,
            FerdieSigner,
            Eve,
            EveSigner,
            Dave,
            DaveSigner,
            treasuryContractFactory,
            treasuryContract,
            psp22Contract,
        };
    }

    function treasuryContractInitParams(sender: any) {
        let treasuryHandler = sender.address;
        return {
            treasuryHandler,
        };
    }

    it.only("Set Treasury handler With An Unathorized handler", async () => {
        let {
           sender,
            BobSigner
        } = await getSigner();

        let {
               treasuryContract,
        } = await setup(sender)
        let initialProposalNonce = await treasuryContract.query.nonce();

        await expect(treasuryContract.tx.setHandler(BobSigner.address, 1048)).to.not.be
            .fulfilled;

        // validate that address has not been updated
        let treasuryHandler = await treasuryContract.query.handler();
        // @ts-ignore
        expect(sender.address.toString()).to.equal(treasuryHandler.output.toString());

        // validate that proposalNonce has not increased
        let proposalNonce = await treasuryContract.query.nonce();
        expect(
            Number(proposalNonce.output) == Number(initialProposalNonce.output)
        ).to.be.true;
    });

    it.only("Set Treasury handler With Invalid Nonce", async () => {
        let {
            sender,
            BobSigner
        } = await getSigner();

        let {
            treasuryContract,
        } = await setup(sender)
        let initialProposalNonce = await treasuryContract.query.nonce();

        // pass 10 as invalid nonce
        await expect(treasuryContract.tx.setHandler(sender.address, 10)).to.not.be
            .fulfilled;

        // validate that address has not been updated
        let treasuryHandler = await treasuryContract.query.handler();
        // @ts-ignore
        expect(sender.address.toString()).to.equal(treasuryHandler.output.toString());

        // validate that proposalNonce has not increased
        let proposalNonce = await treasuryContract.query.nonce();
        expect(
            Number(proposalNonce.output) == Number(initialProposalNonce.output)
        ).to.be.true;
    });

    it.only("Rescue tokens with invalid address", async () => {
        let {
            sender,
            BobSigner,
            EveSigner,
        } = await getSigner();

        // make Eve signer an handler
        let {
            treasuryContract,
        } = await setup(EveSigner.address)

        let initialProposalNonce = await treasuryContract.query.nonce();

        let oldContractBalance =
            await treasuryContract.query.nativeContractBalance();

        // pass null to address
        await expect(
            treasuryContract.tx.rescueTokens(null, null, 100, 1048, {
                value: 1000,
            })
        ).to.not.be.fulfilled;

        // validate contract balance is still the same
        let newContractBalance =
            await treasuryContract.query.nativeContractBalance();
        expect(
            Number(oldContractBalance.output) == Number(newContractBalance.output)
        ).to.be.true;

        // validate that proposalNonce has not increased
        let newProposalNonce = await treasuryContract.query.nonce();
        expect(
            Number(newProposalNonce.output) == Number(initialProposalNonce.output)
        ).to.be.true;
    });

    it.only("Rescue tokens with unauthorized handler", async () => {
        let {
            sender,
            BobSigner,
            EveSigner,
        } = await getSigner();

        // make Eve signer an handler
        let {
            treasuryContract,
        } = await setup(EveSigner.address)

        let initialProposalNonce = await treasuryContract.query.nonce();

        let oldContractBalance =
            await treasuryContract.query.nativeContractBalance();

        await expect(
            treasuryContract.tx.rescueTokens(null, BobSigner.address, 100, 1048, {
                value: 1000,
            })
        ).to.not.be.fulfilled;

        // validate contract balance is still the same
        let newContractBalance =
            await treasuryContract.query.nativeContractBalance();
        expect(
            Number(oldContractBalance.output) == Number(newContractBalance.output)
        ).to.be.true;

        // validate that proposalNonce has not increased
        let newProposalNonce = await treasuryContract.query.nonce();
        expect(
            Number(newProposalNonce.output) == Number(initialProposalNonce.output)
        ).to.be.true;
    });

    it.only("Rescue tokens with invalid amount", async () => {
        let {
            sender,
            BobSigner,
            EveSigner,
        } = await getSigner();

        // make Eve signer an handler
        let {
            treasuryContract,
        } = await setup(EveSigner.address)

        let initialProposalNonce = await treasuryContract.query.nonce();

        let oldContractBalance =
            await treasuryContract.query.nativeContractBalance();

        // pass 0 as invalid amount
        await expect(
            treasuryContract.tx.rescueTokens(null, BobSigner.address, 100, 1048, {
                value: 0,
            })
        ).to.not.be.fulfilled;

        // validate contract balance is still the same
        let newContractBalance =
            await treasuryContract.query.nativeContractBalance();
        expect(
            Number(oldContractBalance.output) == Number(newContractBalance.output)
        ).to.be.true;

        // validate that proposalNonce has not increased
        let newProposalNonce = await treasuryContract.query.nonce();
        expect(
            Number(newProposalNonce.output) == Number(initialProposalNonce.output)
        ).to.be.true;
    });

    it.only("Rescue tokens with invalid nonce", async () => {
        let {
            sender,
            BobSigner,
            EveSigner,
        } = await getSigner();

        // make Eve signer an handler
        let {
            treasuryContract,
        } = await setup(EveSigner.address)

        let initialProposalNonce = await treasuryContract.query.nonce();

        let oldContractBalance =
            await treasuryContract.query.nativeContractBalance();

        // pass 10 as invalid nonce
        await expect(
            treasuryContract.tx.rescueTokens(null, BobSigner.address, 100, 10, {
                value: 0,
            })
        ).to.not.be.fulfilled;

        // validate contract balance is still the same
        let newContractBalance =
            await treasuryContract.query.nativeContractBalance();
        expect(
            Number(oldContractBalance.output) == Number(newContractBalance.output)
        ).to.be.true;

        // validate that proposalNonce has not increased
        let newProposalNonce = await treasuryContract.query.nonce();
        expect(
            Number(newProposalNonce.output) == Number(initialProposalNonce.output)
        ).to.be.true;
    });
});
