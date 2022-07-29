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
import EC from 'elliptic';
import {bytesToHex} from "@noble/hashes/utils";
const ec = new EC.ec('secp256k1');

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

export function normalizeBalance(balance: number): BN {
    return new BN(balance).mul(
        new BN(10).pow(new BN(api.registry.chainDecimals[0]))
    );
}

describe("signature-bridge-tests", () => {
    let signatureBridgeContract: any;
    let signatureBridgeContractFactory: any;
    let sender: any;
    let BobSigner: any;
    let CharlieSigner: any;
    let FerdieSigner: any;
    let EveSigner: any;
    let DaveSigner: any;
    let psp22Contract: any;
    let childProcess: any;
    let privateKey: any;
    let tokenWrapperContract: any;
    let tokenWrapperHandlerContract: any;
    after(() => {
        killContractNode(childProcess);
        return api.disconnect();
    });

    before(async () => {
       childProcess = await startContractNode();
        await api.isReady;
    });

    beforeEach(async () => {
        ({
            signatureBridgeContract,
            signatureBridgeContractFactory,
            sender,
            BobSigner,
            CharlieSigner,
            FerdieSigner,
            EveSigner,
            DaveSigner,
            psp22Contract,
            tokenWrapperContract,
            tokenWrapperHandlerContract,
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
        )

        let keypair = ec.genKeyPair();
        const publicKey = keypair.getPublic('array');
        const privateKey = keypair.getPrivate('hex');

        let publicKeyArray =  Array.from(publicKey)

        console.log(`public key arr is ${toHexString(publicKeyArray)}`)

        // signature bridge instantiation
        const sigBridgeContractFactory = await getContractFactory(
            "signature_bridge",
            CharlieSigner.address
        );

        console.log("finished creating factory for signature bridge")

        const signatureBridgeContract = await sigBridgeContractFactory.deploy(
            "new",
            publicKeyArray,
            0,
            1
        );

        console.log("finished deploying signature bridge")

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

        // token wrapper instantiation
        const tokenWrapperContractFactory = await getContractFactory(
            "governed_token_wrapper",
            CharlieSigner.address
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

        const tokenWrapperHandlerContractFactory = await getContractFactory(
            "token_wrapper_handler",
            sender.address
        );

        const randomVersion = Math.floor(Math.random() * 10000);

        const tokenWrapperHandlerContract = await tokenWrapperHandlerContractFactory.deploy(
            "new",
            signatureBridgeContract.address,
            null,
            null,
            randomVersion,
            tokenWrapperContract.abi.info.source.wasmHash,
            {   tokenName,
                tokenSymbol,
                decimal,
                contractGovernor,
                feeRecipient,
                feePercentage,
                isNativeAllowed,
                wrappingLimit,
                contractProposalNonce,
                totalSupply
            }
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
            signatureBridgeContractFactory,
            signatureBridgeContract,
            psp22Contract,
            privateKey,
            tokenWrapperContract,
            tokenWrapperHandlerContract
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

    it.only("signature bridge test for token handler set resource", async () => {
        let resourceId = Array.from(genResourceId(psp22Contract.address));

        let functionSig =
            await tokenWrapperHandlerContract.query.getSetFeeFunctionSignature();

        let parsedFunctionSig = JSON.parse(functionSig.output).ok
        console.log(parsedFunctionSig);

        let nonce = [0,0,0,1]

        let dataResult =
            await signatureBridgeContract.query.constructData(
                resourceId,
                functionSig,
                nonce,
                resourceId,
                tokenWrapperHandlerContract.address,
                tokenWrapperContract.address
            );

        let data = JSON.parse(dataResult.output).ok;

        console.log(data)
    });

});
