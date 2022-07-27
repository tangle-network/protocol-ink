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

describe("treasury-handler-tests", () => {
  let treasuryHandlerContract: any;
  let treasuryHandlerContractFactory: any;
  let sender: any;
  let BobSigner: any;
  let CharlieSigner: any;
  let FerdieSigner: any;
  let EveSigner: any;
  let DaveSigner: any;
  let psp22Contract: any;
  let treasuryContract: any;
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
      treasuryHandlerContract,
      treasuryHandlerContractFactory,
      sender,
      BobSigner,
      CharlieSigner,
      FerdieSigner,
      EveSigner,
      DaveSigner,
      psp22Contract,
      treasuryContract,
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
    const treasuryContractFactory = await getContractFactory(
      "treasury",
      sender.address
    );

    const treasuryContract = await treasuryContractFactory.deploy(
      "new",
      sender.address
    );

    // token wrapper instantiation
    const treasuryHandlerContractFactory = await getContractFactory(
      "treasury_handler",
      sender.address
    );

    const initialResourceIds = Array.from(
      hexToU8a("0x00000000000000000000000000000000")
    );

    const initialContractAddresses = [psp22Contract.address];
    const randomVersion = Math.floor(Math.random() * 10000);

    treasuryHandlerContractFactory.connect(sender);

    //TODO will change Alice address to signature bridge address as soon as signature bridge is ready
    const treasuryHandlerContract = await treasuryHandlerContractFactory.deploy(
      "new",
      sender.address,
      null,
      null,
      randomVersion,
      treasuryContract.abi.info.source.wasmHash
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
      treasuryHandlerContractFactory,
      treasuryHandlerContract,
      psp22Contract,
      treasuryContract,
    };
  }

  function treasuryContractInitParams(sender: any) {
    let treasuryHandler = sender.address;
    return {
      treasuryHandler,
    };
  }

  it("Migrate Bridge", async () => {
    let initialBridgeAddress =
      await treasuryHandlerContract.query.getBridgeAddress();

    await expect(treasuryHandlerContract.tx.migrateBridge(BobSigner.address)).to
      .be.fulfilled;

    // validate that signature bridge address has been updated
    let newBridgeAddress =
      await treasuryHandlerContract.query.getBridgeAddress();
    console.log(`new bridge address ${newBridgeAddress.output}`);
    expect(initialBridgeAddress.output).to.not.equal(newBridgeAddress.output);
  });

  it("Set Resource", async () => {
    let resourceId = Array.from(genResourceId(psp22Contract.address));

    await expect(
      treasuryHandlerContract.tx.setResource(resourceId, psp22Contract.address)
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult = await treasuryHandlerContract.query.getResourceId(
      psp22Contract.address
    );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    // validate that contract address exists
    let contractAddressResult =
      await treasuryHandlerContract.query.getContractAddress(resourceId);
    expect(psp22Contract.address).to.equal(
      `${JSON.parse(contractAddressResult.output).ok}`
    );

    // validate that contract address is whitelisted
    let isContractWhitelistedResult =
      await treasuryHandlerContract.query.isContractAddressWhitelisted(
        psp22Contract.address
      );
    expect(JSON.parse(isContractWhitelistedResult.output).ok).to.be.true;
  });

  it("Execute Proposal for setting handler", async () => {
    // sets random resource
    let resourceId = Array.from(genResourceId(psp22Contract.address));
    await expect(
      treasuryHandlerContract.tx.setResource(resourceId, psp22Contract.address)
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult = await treasuryHandlerContract.query.getResourceId(
      psp22Contract.address
    );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    let functionSig = [29, 246, 193, 234];

    let nonce = [0, 0, 4, 24];

    let dataResult =
      await treasuryHandlerContract.query.constructDataForSetHandler(
        resourceId,
        functionSig,
        nonce,
        sender.address
      );

    await expect(
      treasuryHandlerContract.tx.executeProposal(
        resourceId,
        JSON.parse(dataResult.output).ok
      )
    ).to.be.fulfilled;
  });

  it("Execute Proposal for rescuing tokens", async () => {
    // sets random resource
    let resourceId = Array.from(genResourceId(psp22Contract.address));

    await expect(
      treasuryHandlerContract.tx.setResource(resourceId, psp22Contract.address)
    ).to.be.fulfilled;

    // validate that resource id exists
    let resourceIdResult = await treasuryHandlerContract.query.getResourceId(
      psp22Contract.address
    );
    expect(`0x${toHexString(resourceId)}`).to.equal(
      `${JSON.parse(resourceIdResult.output).ok}`
    );

    let functionSig = [173, 65, 206, 122];

    let nonce = [0, 0, 4, 24];

    let amountToRescue = [0, 0, 0, 100];

    let dataResult =
      await treasuryHandlerContract.query.constructDataForRescueTokens(
        resourceId,
        functionSig,
        nonce,
        null,
        BobSigner.address,
        amountToRescue
      );

    await expect(
      treasuryHandlerContract.tx.executeProposal(
        resourceId,
        JSON.parse(dataResult.output).ok,
        {
          value: 1000,
        }
      )
    ).to.be.fulfilled;
  });
});
