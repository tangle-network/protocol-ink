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
    genResourceId, encodeExtData,
} from "./util";
import {hexToU8a, u8aToHex} from "@polkadot/util";
import { createType } from "@polkadot/types";
import keccak256 from "keccak256";
import { BigNumber, BigNumberish } from "ethers";
import { registry } from "@subsocial/types";
import {decodeAddress, naclEncrypt, randomAsU8a} from '@polkadot/util-crypto';
import child from "child_process";
import path from "path";
import fs from 'fs';
import {
    Note,
    ProvingManagerSetupInput,
    ArkworksProvingManager,
    Utxo,
    VAnchorProof, toFixedHex,
} from '@webb-tools/sdk-core';
import {hex} from "@scure/base";

import {bytesToHex} from "@noble/hashes/utils";


const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

export function normalizeBalance(balance: number): BN {
    return new BN(balance).mul(
        new BN(10).pow(new BN(api.registry.chainDecimals[0]))
    );
}

describe("vanchor-tests", () => {
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
            vAnchorContract,
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
            maxFee,
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

        // Poseidon instantiation
        const poseidonContractFactory = await getContractFactory(
            "poseidon",
            sender.address
        );
        const poseidonContract = await poseidonContractFactory.deploy("new");

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
                totalSupply,
            },
            randomVersion + 1,
            poseidonContract.abi.info.source.wasmHash,
            tokenWrapperContract.abi.info.source.wasmHash
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
            anchorHandlerContractFactory,
            anchorHandlerContract,
            psp22Contract,
            tokenWrapperContract,
            vAnchorContract,
        };
    }

    async function vAnchorContractInitParams() {
        let maxEdges = 2;
        let chainId = 1;
        let levels = 10;
        let maxDepositAmount = currencyToUnitI128(20);;
        let minWithdrwalAmount = 0;
        let maxExtAmt = currencyToUnitI128(20);
        let maxFee = 100;

        return {
            maxEdges,
            chainId,
            levels,
            maxDepositAmount,
            minWithdrwalAmount,
            maxExtAmt,
            maxFee,
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

    it.only("Transact Deposit Test", async () => {
        const {
            levels,
        } = await vAnchorContractInitParams();
        const chainId = '6597069766657';
        const outputChainId = BigInt(chainId);
        const secret = randomAsU8a();
        const gitRoot = child
            .execSync('git rev-parse --show-toplevel')
            .toString()
            .trim();

        const pkPath = path.join(
            // tests path
            gitRoot,
            'tests',
            'protocol-substrate-fixtures',
            'vanchor',
            'bn254',
            'x5',
            '2-2-2',
            'proving_key_uncompressed.bin'
        );
        const pk_hex = fs.readFileSync(pkPath).toString('hex');
        const pk = hexToU8a(pk_hex);

        // Creating two empty vanchor notes
        const note1 = await generateVAnchorNote(
            0,
            Number(outputChainId.toString()),
            Number(outputChainId.toString()),
            0
        );
        const note2 = await note1.getDefaultUtxoNote();
        const publicAmount = currencyToUnitI128(10);
        const notes = [note1, note2];
        // Output UTXOs configs
        const output1 = await Utxo.generateUtxo({
            curve: 'Bn254',
            backend: 'Arkworks',
            amount: publicAmount.toString(),
            chainId,
        });
        const output2 = await Utxo.generateUtxo({
            curve: 'Bn254',
            backend: 'Arkworks',
            amount: '0',
            chainId,
        });

        // Configure a new proving manager with direct call
        const provingManager = new ArkworksProvingManager(null);
        const leavesMap: any = {};

        const address = sender.address;
        const extAmount = currencyToUnitI128(10);
        const fee = 0;

        let rootsResult = await vAnchorContract.query.customRootsFor2(levels);
        // @ts-ignore
        console.log(`roots are ${rootsResult.output}`);
        console.log(`root is ${hexToU8a(rootsResult.output[0].toString())}`);
        // @ts-ignore
        let roots = rootsResult.output;

        const rootsSet = [hexToU8a(roots[0].toString().replace('0x', '')), hexToU8a(roots[1].toString().replace('0x', ''))];
        const decodedAddress = decodeAddress(address);
        console.log(`ext amount ${extAmount.toString()}`)
        const { encrypted: comEnc1 } = naclEncrypt(output1.commitment, secret);
        const { encrypted: comEnc2 } = naclEncrypt(output2.commitment, secret);

        const setup: ProvingManagerSetupInput<'vanchor'> = {
            chainId: outputChainId.toString(),
            indices: [0, 0],
            inputNotes: notes,
            leavesMap: leavesMap,
            output: [output1, output2],
            encryptedCommitments: [comEnc1, comEnc2],
            provingKey: pk,
            publicAmount: String(publicAmount),
            roots: rootsSet,
            relayer: decodedAddress,
            recipient: decodedAddress,
            extAmount: extAmount.toString(),
            fee: fee.toString(),
        };

        const data = (await provingManager.prove('vanchor', setup)) as VAnchorProof;

        const extData = {
            recipient: address,
            relayer: address,
            extAmount: extAmount,
            fee,
            encryptedOutput1: u8aToHex(comEnc1),
            encryptedOutput2: u8aToHex(comEnc2),
        };

        let proofData = {
            proof: `0x${data.proof}`,
            publicAmount: data.publicAmount,
            roots: rootsSet,
            inputNullifiers: data.inputUtxos.map((input) => `0x${input.nullifier}`),
            outputCommitments: data.outputNotes.map((note) =>
                u8aToHex(note.note.getLeafCommitment())
            ),
            extDataHash: data.extDataHash,
        };

        await expect(vAnchorContract.tx.transactDeposit(proofData, extData, tokenWrapperContract.address, extAmount)).to
          .be.fulfilled;


    });

    it.only("Transact Withdrawal Test", async () => {
        const {
            levels,
        } = await vAnchorContractInitParams();
        const chainId = '6597069766657';
        const outputChainId = BigInt(chainId);
        const secret = randomAsU8a();
        const gitRoot = child
            .execSync('git rev-parse --show-toplevel')
            .toString()
            .trim();

        const pkPath = path.join(
            // tests path
            gitRoot,
            'tests',
            'protocol-substrate-fixtures',
            'vanchor',
            'bn254',
            'x5',
            '2-2-2',
            'proving_key_uncompressed.bin'
        );
        const pk_hex = fs.readFileSync(pkPath).toString('hex');
        const pk = hexToU8a(pk_hex);

        // Creating two empty vanchor notes
        const note1 = await generateVAnchorNote(
            0,
            Number(outputChainId.toString()),
            Number(outputChainId.toString()),
            0
        );
        const note2 = await note1.getDefaultUtxoNote();
        const publicAmount = currencyToUnitI128(10);
        const notes = [note1, note2];
        // Output UTXOs configs
        const output1 = await Utxo.generateUtxo({
            curve: 'Bn254',
            backend: 'Arkworks',
            amount: publicAmount.toString(),
            chainId,
        });
        const output2 = await Utxo.generateUtxo({
            curve: 'Bn254',
            backend: 'Arkworks',
            amount: '0',
            chainId,
        });

        // Configure a new proving manager with direct call
        const provingManager = new ArkworksProvingManager(null);
        const leavesMap: any = {};

        const address = sender.address;
        const extAmount = currencyToUnitI128(10);
        const fee = 0;

        let rootsResult = await vAnchorContract.query.customRootsFor2(levels);
        // @ts-ignore
        console.log(`roots are ${rootsResult.output}`);
        console.log(`root is ${hexToU8a(rootsResult.output[0].toString())}`);
        // @ts-ignore
        let roots = rootsResult.output;

        const rootsSet = [hexToU8a(roots[0].toString().replace('0x', '')), hexToU8a(roots[1].toString().replace('0x', ''))];
        const decodedAddress = decodeAddress(address);
        console.log(`ext amount ${extAmount.toString()}`)
        const { encrypted: comEnc1 } = naclEncrypt(output1.commitment, secret);
        const { encrypted: comEnc2 } = naclEncrypt(output2.commitment, secret);

        const setup: ProvingManagerSetupInput<'vanchor'> = {
            chainId: outputChainId.toString(),
            indices: [0, 0],
            inputNotes: notes,
            leavesMap: leavesMap,
            output: [output1, output2],
            encryptedCommitments: [comEnc1, comEnc2],
            provingKey: pk,
            publicAmount: String(publicAmount),
            roots: rootsSet,
            relayer: decodedAddress,
            recipient: decodedAddress,
            extAmount: extAmount.toString(),
            fee: fee.toString(),
        };

        const data = (await provingManager.prove('vanchor', setup)) as VAnchorProof;

        const extData = {
            recipient: address,
            relayer: address,
            extAmount: extAmount,
            fee,
            encryptedOutput1: u8aToHex(comEnc1),
            encryptedOutput2: u8aToHex(comEnc2),
        };

        let proofData = {
            proof: `0x${data.proof}`,
            publicAmount: data.publicAmount,
            roots: rootsSet,
            inputNullifiers: data.inputUtxos.map((input) => `0x${input.nullifier}`),
            outputCommitments: data.outputNotes.map((note) =>
                u8aToHex(note.note.getLeafCommitment())
            ),
            extDataHash: data.extDataHash,
        };


        await expect(vAnchorContract.tx.transactWithdraw(proofData, extData)).to
            .be.fulfilled;

    });
});

async function generateVAnchorNote(
    amount: number,
    chainId: number,
    outputChainId: number,
    index?: number
) {
    const note = await Note.generateNote({
        amount: String(amount),
        backend: 'Arkworks',
        curve: 'Bn254',
        denomination: String(18),
        exponentiation: String(5),
        hashFunction: 'Poseidon',
        index,
        protocol: 'vanchor',
        sourceChain: String(chainId),
        sourceIdentifyingData: '1',
        targetChain: String(outputChainId),
        targetIdentifyingData: '1',
        tokenSymbol: 'WEBB',
        version: 'v2',
        width: String(5),
    });

    return note;
}

function currencyToUnitI128(currencyAmount: number) {
    let bn = BigNumber.from(currencyAmount);
    return bn.mul(1_000_000_000_000);
}
