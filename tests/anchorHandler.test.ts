import { expect } from "chai";
import { artifacts, network, patract } from "redspot";
import BN from "bn.js";
import {
    killContractNode,
    startContractNode,
    toHexString,
    parseHexString,
    hexStringToByteArray,
    toEncodedBinary,
    genResourceId,
} from "./util";
import { hexToU8a } from "@polkadot/util";
import { createType } from "@polkadot/types";
import keccak256 from "keccak256";
import { BigNumber, BigNumberish } from "ethers";
import { registry } from "@subsocial/types";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

export function normalizeBalance(balance: number): BN {
    return new BN(balance).mul(
        new BN(10).pow(new BN(api.registry.chainDecimals[0]))
    );
}

describe("anchor-handler-tests", () => {
    let anchorHandlerContract: any;
    let anchorHandlerContractFactory: any;
    let sender: any;
    let BobSigner: any;
    let CharlieSigner: any;
    let FerdieSigner: any;
    let EveSigner: any;
    let DaveSigner: any;
    let psp22Contract: any;
    let tokenWrapperContract: any;
    let vAnchorContract: any;
    let childProcess: any;
    after(() => {
        //killContractNode(childProcess);
        return api.disconnect();
    });

    before(async () => {
        //childProcess = await startContractNode();
        await api.isReady;
    });

    beforeEach(async () => {
        ({
            anchorHandlerContract,
            anchorHandlerContractFactory,
            sender,
            BobSigner,
            CharlieSigner,
            FerdieSigner,
            EveSigner,
            DaveSigner,
            psp22Contract,
            tokenWrapperContract,
            vAnchorContract
        } = await setup());
    });

    async function setup() {
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
        const {
            maxEdges,
            chainId,
            levels,
            maxDepositAmount,
            minWithdrwalAmount,
            maxExtAmt,
            maxFee
        } = await vAnchorContractInitParams();

        const {
            tokenName,
            tokenSymbol,
            decimal,
            contractGovernor,
            feeRecipient,
            feePercentage,
            isNativeAllowed,
            wrappingLimit,
            contractProposalNonce,
            totalSupply,
            governorBalance,
        } = await tokenWrapperContractInitParams(
            sender,
            BobSigner,
            CharlieSigner,
            DaveSigner
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

        const tokenWrapperContractFactory = await getContractFactory(
            "governed_token_wrapper",
            sender.address
        );

        const tokenWrapperContract = await tokenWrapperContractFactory.deploy(
            "new",
            tokenName,
            tokenSymbol,
            decimal,
            contractGovernor,
            feeRecipient,
            feePercentage,
            isNativeAllowed,
            wrappingLimit,
            contractProposalNonce,
            totalSupply
        );

        console.log(`tokenWrapper ${tokenWrapperContract.abi.info.source.wasmHash}`)
        console.log("finished deploying tokenwrapper")

        // Poseidon instantiation
        const poseidonContractFactory = await getContractFactory(
            "poseidon",
            sender.address
        );
        const poseidonContract = await poseidonContractFactory.deploy("new");


        console.log(`poseidon ${poseidonContract.abi.info.source.wasmHash}`)


        console.log("finished deploying poseidon")



        const randomVersion = Math.floor(Math.random() * 10000);

        const vAnchorContractFactory = await getContractFactory(
            "vanchor",
            sender.address
        );

        const vAnchorContract = await vAnchorContractFactory.deploy(
            "new",
            maxEdges,
            chainId,
            levels,
            maxDepositAmount,
            minWithdrwalAmount,
            maxExtAmt,
            maxFee,
            tokenWrapperContract.address,
            sender.address,
            {
                tokenName,
                tokenSymbol,
                decimal,
                contractGovernor,
                feeRecipient,
                feePercentage,
                isNativeAllowed,
                wrappingLimit,
                contractProposalNonce,
                totalSupply
            },
            randomVersion+1,
            poseidonContract.abi.info.source.wasmHash,
            tokenWrapperContract.abi.info.source.wasmHash
        );

        console.log(`vanchro ${vAnchorContract.abi.info.source.wasmHash}`)

        console.log("finished deploying vanchor")

        // token wrapper instantiation
        const anchorHandlerContractFactory = await getContractFactory(
            "anchor_handler",
            sender.address
        );

        const initialResourceIds = Array.from(
            hexToU8a("0x00000000000000000000000000000000")
        );

        const tokenWrapperContractAddress = tokenWrapperContract.address;
        const senderAddress = sender.address;
        const poseidonContractHash = poseidonContract.abi.info.source.wasmHash;
        //const verifierContractHash =   vAnchorVerifierContract.abi.info.source.wasmHash;
        const tokenWrapperContractHash =   tokenWrapperContract.abi.info.source.wasmHash;

        console.log(`sender address ${senderAddress}`);
        //console.log(`sender address ${hexToU8a(senderAddress)}`);

        //TODO will change Alice address to signature bridge address as soon as signature bridge is ready
        // @ts-ignore
        // @ts-ignore
        // @ts-ignore
        const anchorHandlerContract =
            await anchorHandlerContractFactory.deploy(
                "new",
                sender.address,
                null,
                null,
                vAnchorContract.abi.info.source.wasmHash,
                {
                    maxEdges,
                    chainId,
                    levels,
                    maxDepositAmount,
                    minWithdrwalAmount,
                    maxExtAmt,
                    maxFee,
                    tokenWrapperContractAddress,
                    handler: sender.address,
                    randomVersion,
                    poseidonContractHash,
                    tokenWrapperContractHash
                },
                {
                    tokenName,
                    tokenSymbol,
                    decimal,
                    contractGovernor,
                    feeRecipient,
                    feePercentage,
                    isNativeAllowed,
                    wrappingLimit,
                    contractProposalNonce,
                    totalSupply,
                }
            );

        console.log("finished deploying handler")

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
            anchorHandlerContractFactory,
            anchorHandlerContract,
            psp22Contract,
            tokenWrapperContract,
            vAnchorContract
        };
    }

    async function vAnchorContractInitParams() {

        let maxEdges = 2;
        let chainId = 1;
        let levels = 30;
        let maxDepositAmount = 1000000;
        let minWithdrwalAmount = 100;
        let maxExtAmt = 100;
        let maxFee = 10;

        return {
            maxEdges,
            chainId,
            levels,
            maxDepositAmount,
            minWithdrwalAmount,
            maxExtAmt,
            maxFee
        };
    }


    async function tokenWrapperContractInitParams(
        sender: any,
        BobSigner: any,
        CharlieSigner: any,
        DaveSigner: any
    ) {
        let tokenName = "Webb";
        let tokenSymbol = "Webb";
        let decimal = 4;
        let contractGovernor = sender.address;
        let feeRecipient = sender.address;
        let feePercentage = 1;
        let isNativeAllowed = true;
        let wrappingLimit = 1_000_000_000_01;
        let contractProposalNonce = 0;
        let tokenAddress = BobSigner.address;
        let totalSupply = 1_000_000_000;
        let governorBalance = 9_000_000;

        return {
            tokenName,
            tokenSymbol,
            decimal,
            contractGovernor,
            feeRecipient,
            feePercentage,
            isNativeAllowed,
            wrappingLimit,
            contractProposalNonce,
            tokenAddress,
            totalSupply,
            governorBalance,
        };
    }

    it("Migrate Bridge", async () => {
        let initialBridgeAddress =
            await anchorHandlerContract.query.getBridgeAddress();

        await expect(
            anchorHandlerContract.tx.migrateBridge(BobSigner.address)
        ).to.be.fulfilled;

        // validate that signature bridge address has been updated
        let newBridgeAddress =
            await anchorHandlerContract.query.getBridgeAddress();
        expect(initialBridgeAddress.output).to.not.equal(newBridgeAddress.output);
    });

    it("Set Resource", async () => {
        let resourceId = Array.from(genResourceId(psp22Contract.address));

        await expect(
            anchorHandlerContract.tx.setResource(
                resourceId,
                psp22Contract.address
            )
        ).to.be.fulfilled;

        // validate that resource id exists
        let resourceIdResult =
            await anchorHandlerContract.query.getResourceId(
                psp22Contract.address
            );
        expect(`0x${toHexString(resourceId)}`).to.equal(
            `${JSON.parse(resourceIdResult.output).ok}`
        );

        // validate that contract address exists
        let contractAddressResult =
            await anchorHandlerContract.query.getContractAddress(resourceId);
        expect(psp22Contract.address).to.equal(
            `${JSON.parse(contractAddressResult.output).ok}`
        );

        // validate that contract address is whitelisted
        let isContractWhitelistedResult =
            await anchorHandlerContract.query.isContractAddressWhitelisted(
                psp22Contract.address
            );
        expect(JSON.parse(isContractWhitelistedResult.output).ok).to.be.true;
    });

    it.only("Execute Proposal for set handler", async () => {

        let initialHandler =
            await vAnchorContract.query.handler();

        console.log(`initial handler ${initialHandler.output}`);

        await expect(
            vAnchorContract.tx.setHandler(
                anchorHandlerContract.address,
                1048
            )
        ).to.be.fulfilled;

        let newHandler =
            await vAnchorContract.query.handler();

        console.log(`new handler ${newHandler.output}`);


        // sets random resource
        let resourceId = Array.from(genResourceId(vAnchorContract.address));
        await expect(
            anchorHandlerContract.tx.setResource(
                resourceId,
                vAnchorContract.address
            )
        ).to.be.fulfilled;

        // validate that resource id exists
        let resourceIdResult =
            await anchorHandlerContract.query.getResourceId(
                vAnchorContract.address
            );
        expect(`0x${toHexString(resourceId)}`).to.equal(
            `${JSON.parse(resourceIdResult.output).ok}`
        );

        let functionSig =
            await anchorHandlerContract.query.getSetHandlerFunctionSignature();

        let parsedFunctionSig = JSON.parse(functionSig.output).ok;

        let nonce = [0, 0, 0, 0, 0, 0, 0, 0];

        let dataResult =
            await anchorHandlerContract.query.constructDataForSetHandler(
                resourceId,
                parsedFunctionSig,
                nonce,
                BobSigner.address
            );

        await expect(
            anchorHandlerContract.tx.executeProposal(
                resourceId,
                JSON.parse(dataResult.output).ok
            )
        ).to.be.fulfilled;
    });

    it("Execute Proposal for update edge", async () => {
        // sets random resource
        let resourceId = Array.from(genResourceId(psp22Contract.address));
        await expect(
            anchorHandlerContract.tx.setResource(
                resourceId,
                psp22Contract.address
            )
        ).to.be.fulfilled;

        // validate that resource id exists
        let resourceIdResult =
            await anchorHandlerContract.query.getResourceId(
                psp22Contract.address
            );
        expect(`0x${toHexString(resourceId)}`).to.equal(
            `${JSON.parse(resourceIdResult.output).ok}`
        );

        let functionSig =
            await anchorHandlerContract.query.getUpdateEdgeFunctionSignature();

        let parsedFunctionSig = JSON.parse(functionSig.output).ok;

        let dataResult =
            await anchorHandlerContract.query.constructDataForUpdateEdge(
                resourceId,
                parsedFunctionSig,
                [0, 0, 0, 0, 0, 0, 0, 1],
                [0, 0, 0, 0, 0, 0, 4, 24, 0, 0, 0, 0, 0, 0, 4, 24, 0, 0, 0, 0, 0, 0, 4, 24, 0, 0, 0, 0, 0, 0, 4, 24],
                [0, 0, 0, 0,],
                [0, 0, 0, 0, 0, 0, 4, 24, 0, 0, 0, 0, 0, 0, 4, 24, 0, 0, 0, 0, 0, 0, 4, 24, 0, 0, 0, 0, 0, 0, 4, 24],
            );

        await expect(
            anchorHandlerContract.tx.executeProposal(
                resourceId,
                JSON.parse(dataResult.output).ok
            )
        ).to.be.fulfilled;
    });

    it("Execute Proposal for configure max deposit limit", async () => {
        // sets random resource
        let resourceId = Array.from(genResourceId(psp22Contract.address));
        await expect(
            anchorHandlerContract.tx.setResource(
                resourceId,
                psp22Contract.address
            )
        ).to.be.fulfilled;

        // validate that resource id exists
        let resourceIdResult =
            await anchorHandlerContract.query.getResourceId(
                psp22Contract.address
            );
        expect(`0x${toHexString(resourceId)}`).to.equal(
            `${JSON.parse(resourceIdResult.output).ok}`
        );

        let functionSig =
            await anchorHandlerContract.query.getConfigureMaxDepositLimitFunctionSignature();

        let parsedFunctionSig = JSON.parse(functionSig.output).ok;

        let amount = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 232];

        let dataResult =
            await anchorHandlerContract.query.constructDataForLimitAmount(
                resourceId,
                parsedFunctionSig,
                amount
            );

        await expect(
            anchorHandlerContract.tx.executeProposal(
                resourceId,
                JSON.parse(dataResult.output).ok
            )
        ).to.be.fulfilled;
    });

    it("Execute Proposal for configure min withdrawal limit", async () => {
        // sets random resource
        let resourceId = Array.from(genResourceId(psp22Contract.address));
        await expect(
            anchorHandlerContract.tx.setResource(
                resourceId,
                psp22Contract.address
            )
        ).to.be.fulfilled;

        // validate that resource id exists
        let resourceIdResult =
            await anchorHandlerContract.query.getResourceId(
                psp22Contract.address
            );
        expect(`0x${toHexString(resourceId)}`).to.equal(
            `${JSON.parse(resourceIdResult.output).ok}`
        );

        let functionSig =
            await anchorHandlerContract.query.getConfigureMinWithdrawalLimitFunctionSignature();

        let parsedFunctionSig = JSON.parse(functionSig.output).ok;

        let amount = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 232];

        let dataResult =
            await anchorHandlerContract.query.constructDataForLimitAmount(
                resourceId,
                parsedFunctionSig,
                amount
            );

        await expect(
            anchorHandlerContract.tx.executeProposal(
                resourceId,
                JSON.parse(dataResult.output).ok
            )
        ).to.be.fulfilled;
    });


});
