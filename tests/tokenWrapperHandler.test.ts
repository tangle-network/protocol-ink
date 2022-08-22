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

describe("token-wrapper-handler-tests", () => {
  let tokenWrapperHandlerContract: any;
  let tokenWrapperHandlerContractFactory: any;
  let sender: any;
  let BobSigner: any;
  let CharlieSigner: any;
  let FerdieSigner: any;
  let EveSigner: any;
  let DaveSigner: any;
  let psp22Contract: any;
  let tokenWrapperContract: any;
  let childProcess: any;
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
      tokenWrapperHandlerContract,
      tokenWrapperHandlerContractFactory,
      sender,
      BobSigner,
      CharlieSigner,
      FerdieSigner,
      EveSigner,
      DaveSigner,
      psp22Contract,
      tokenWrapperContract,
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

    // Poseidon instantiation
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

    // token wrapper instantiation
    const tokenWrapperHandlerContractFactory = await getContractFactory(
      "token_wrapper_handler",
      sender.address
    );

    const initialResourceIds = Array.from(
      hexToU8a("0x00000000000000000000000000000000")
    );

    const initialContractAddresses = [psp22Contract.address];
    const randomVersion = Math.floor(Math.random() * 10000);

    //TODO will change Alice address to signature bridge address as soon as signature bridge is ready
    const tokenWrapperHandlerContract =
      await tokenWrapperHandlerContractFactory.deploy(
        "new",
        sender.address,
        null,
        null,
        randomVersion,
        tokenWrapperContract.abi.info.source.wasmHash,
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
      tokenWrapperHandlerContractFactory,
      tokenWrapperHandlerContract,
      psp22Contract,
      tokenWrapperContract,
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
      await tokenWrapperHandlerContract.query.getBridgeAddress();

    await expect(
      tokenWrapperHandlerContract.tx.migrateBridge(BobSigner.address)
    ).to.be.fulfilled;

    // validate that signature bridge address has been updated
    let newBridgeAddress =
      await tokenWrapperHandlerContract.query.getBridgeAddress();
    expect(initialBridgeAddress.output).to.not.equal(newBridgeAddress.output);
  });

  it("Set Resource", async () => {
    let resourceId = Array.from(genResourceId(psp22Contract.address));

    await expect(
      tokenWrapperHandlerContract.tx.setResource(
        resourceId,
        psp22Contract.address
      )
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult =
      await tokenWrapperHandlerContract.query.getResourceId(
        psp22Contract.address
      );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    // validate that contract address exists
    let contractAddressResult =
      await tokenWrapperHandlerContract.query.getContractAddress(resourceId);
    expect(psp22Contract.address).to.equal(
      `${JSON.parse(contractAddressResult.output).ok}`
    );

    // validate that contract address is whitelisted
    let isContractWhitelistedResult =
      await tokenWrapperHandlerContract.query.isContractAddressWhitelisted(
        psp22Contract.address
      );
    expect(JSON.parse(isContractWhitelistedResult.output).ok).to.be.true;
  });

  it("Execute Proposal for setting fee", async () => {
    // sets random resource
    let resourceId = Array.from(genResourceId(psp22Contract.address));
    await expect(
      tokenWrapperHandlerContract.tx.setResource(
        resourceId,
        psp22Contract.address
      )
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult =
      await tokenWrapperHandlerContract.query.getResourceId(
        psp22Contract.address
      );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    let functionSig =
      await tokenWrapperHandlerContract.query.getSetFeeFunctionSignature();

    let parsedFunctionSig = JSON.parse(functionSig.output).ok;

    let nonce = 1;

    let amount = 100;

    let dataResult =
      await tokenWrapperHandlerContract.query.constructDataForSetFee(
        resourceId,
        parsedFunctionSig,
        nonce,
        amount
      );

    await expect(
      tokenWrapperHandlerContract.tx.executeProposal(
        resourceId,
        JSON.parse(dataResult.output).ok
      )
    ).to.be.fulfilled;
  });

  it("Execute Proposal for setting fee recipient", async () => {
    // sets random resource
    let resourceId = Array.from(genResourceId(psp22Contract.address));
    await expect(
      tokenWrapperHandlerContract.tx.setResource(
        resourceId,
        psp22Contract.address
      )
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult =
      await tokenWrapperHandlerContract.query.getResourceId(
        psp22Contract.address
      );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    let functionSig =
      await tokenWrapperHandlerContract.query.getSetFeeRecipientFunctionSignature();

    let parsedFunctionSig = JSON.parse(functionSig.output).ok;

    let nonce = 1;

    let dataResult = await tokenWrapperHandlerContract.query.constructData(
      resourceId,
      parsedFunctionSig,
      nonce,
      BobSigner.address
    );

    await expect(
      tokenWrapperHandlerContract.tx.executeProposal(
        resourceId,
        JSON.parse(dataResult.output).ok
      )
    ).to.be.fulfilled;
  });

  it("Execute Proposal for add token address", async () => {
    // sets random resource
    let resourceId = Array.from(genResourceId(psp22Contract.address));
    await expect(
      tokenWrapperHandlerContract.tx.setResource(
        resourceId,
        psp22Contract.address
      )
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult =
      await tokenWrapperHandlerContract.query.getResourceId(
        psp22Contract.address
      );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    let functionSig =
      await tokenWrapperHandlerContract.query.getAddTokenAddressFunctionSignature();

    let parsedFunctionSig = JSON.parse(functionSig.output).ok;

    let nonce = 1;

    let dataResult = await tokenWrapperHandlerContract.query.constructData(
      resourceId,
      parsedFunctionSig,
      nonce,
      psp22Contract.address
    );

    await expect(
      tokenWrapperHandlerContract.tx.executeProposal(
        resourceId,
        JSON.parse(dataResult.output).ok
      )
    ).to.be.fulfilled;
  });

  it("Execute Proposal for remove token address", async () => {
    // sets random resource
    let resourceId = Array.from(genResourceId(psp22Contract.address));
    await expect(
      tokenWrapperHandlerContract.tx.setResource(
        resourceId,
        psp22Contract.address
      )
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult =
      await tokenWrapperHandlerContract.query.getResourceId(
        psp22Contract.address
      );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    // First add token address

    let functionSig =
      await tokenWrapperHandlerContract.query.getAddTokenAddressFunctionSignature();

    let parsedFunctionSig = JSON.parse(functionSig.output).ok;

    let nonce = 1;

    let dataResult = await tokenWrapperHandlerContract.query.constructData(
      resourceId,
      parsedFunctionSig,
      nonce,
      psp22Contract.address
    );

    await expect(
      tokenWrapperHandlerContract.tx.executeProposal(
        resourceId,
        JSON.parse(dataResult.output).ok
      )
    ).to.be.fulfilled;

    // Then remove token address
    functionSig =
      await tokenWrapperHandlerContract.query.getRemoveTokenAddressFunctionSignature();

    parsedFunctionSig = JSON.parse(functionSig.output).ok;

    nonce = 2;

    dataResult = await tokenWrapperHandlerContract.query.constructData(
      resourceId,
      parsedFunctionSig,
      nonce,
      psp22Contract.address
    );

    await expect(
      tokenWrapperHandlerContract.tx.executeProposal(
        resourceId,
        JSON.parse(dataResult.output).ok
      )
    ).to.be.fulfilled;
  });
});
