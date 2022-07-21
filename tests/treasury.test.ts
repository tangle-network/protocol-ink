import { expect } from "chai";
import { artifacts, network, patract } from "redspot";
import BN from "bn.js";
import { killContractNode, startContractNode } from "./util";

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
      treasuryContract,
      treasuryContractFactory,
      sender,
      BobSigner,
      CharlieSigner,
      FerdieSigner,
      EveSigner,
      DaveSigner,
      psp22Contract,
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

    // token wrapper instantiation
    const treasuryContractFactory = await getContractFactory(
      "treasury",
      sender.address
    );

    const treasuryContract = await treasuryContractFactory.deploy(
      "new",
      sender.address
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

  it("Set Treasury handler", async () => {
    let initialProposalNonce = await treasuryContract.query.nonce();

    await expect(treasuryContract.tx.setHandler(BobSigner.address, 1048)).to.be
      .fulfilled;

    // validate that address has been updated
    let newTreasuryHandler = await treasuryContract.query.handler();
    expect(sender.address).to.not.equal(newTreasuryHandler.output);

    // validate that proposalNonce has increased
    let newProposalNonce = await treasuryContract.query.nonce();
    expect(
      Number(newProposalNonce.output) > Number(initialProposalNonce.output)
    ).to.be.true;
  });

  it("Rescue native tokens when amount to rescue is less than contract balance", async () => {
    let initialProposalNonce = await treasuryContract.query.nonce();

    let oldContractBalance =
      await treasuryContract.query.nativeContractBalance();

    await expect(
      treasuryContract.tx.rescueTokens(null, BobSigner.address, 100, 1048, {
        value: 1000,
      })
    ).to.be.fulfilled;

    // validate contract balance
    let newContractBalance =
      await treasuryContract.query.nativeContractBalance();
    expect(
      Number(oldContractBalance.output) < Number(newContractBalance.output)
    ).to.be.true;

    // validate that proposalNonce has increased
    let newProposalNonce = await treasuryContract.query.nonce();
    expect(
      Number(newProposalNonce.output) > Number(initialProposalNonce.output)
    ).to.be.true;
  });

  it("Rescue native tokens when amount to rescue is greater than contract balance", async () => {
    let initialProposalNonce = await treasuryContract.query.nonce();

    let oldContractBalance =
      await treasuryContract.query.nativeContractBalance();

    await expect(
      treasuryContract.tx.rescueTokens(null, BobSigner.address, 2000, 1048, {
        value: 1000,
      })
    ).to.be.fulfilled;

    // validate contract balance, contract balance will still be zero
    let newContractBalance =
      await treasuryContract.query.nativeContractBalance();
    expect(
      Number(oldContractBalance.output) == Number(newContractBalance.output)
    ).to.be.true;

    // validate that proposalNonce has increased
    let newProposalNonce = await treasuryContract.query.nonce();
    expect(
      Number(newProposalNonce.output) > Number(initialProposalNonce.output)
    ).to.be.true;
  });

  it("Rescue psp22 tokens when amount to rescue is less than contract balance", async () => {
    let initialProposalNonce = await treasuryContract.query.nonce();

    let oldContractBalance =
      await treasuryContract.query.psp22ContractBalance();
    let oldBobPsp22Balance = await treasuryContract.query.psp22Balance(
      BobSigner.address
    );

    // secondly set psp22 allowance. Make psp22Contract the owner, and Alice the spender
    let allowedAmount = 500000;
    expect(
      await treasuryContract.tx.setPsp22AllowanceForOwner(
        psp22Contract.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await treasuryContract.query.getPsp22Allowance(
      psp22Contract.address,
      sender.address
    );

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output) === allowedAmount);

    await expect(
      treasuryContract.tx.insertPsp22Balance(psp22Contract.address, 2000)
    ).to.be.fulfilled;

    await expect(
      treasuryContract.tx.insertPsp22Balance(treasuryContract.address, 2000)
    ).to.be.fulfilled;

    // perform token rescue for amount lesser than contract balance
    await expect(
      treasuryContract.tx.rescueTokens(
        psp22Contract.address,
        BobSigner.address,
        100,
        1048
      )
    ).to.be.fulfilled;

    let psp22ContractBalance =
      await treasuryContract.query.psp22ContractBalance(psp22Contract.address);
    // validate Bob's psp22 balance
    let newBobPsp22Balance = await treasuryContract.query.psp22Balance(
      BobSigner.address
    );
    expect(
      Number(oldBobPsp22Balance.output) != Number(newBobPsp22Balance.output)
    ).to.be.true;

    // validate contract balance
    let newContractBalance =
      await treasuryContract.query.psp22ContractBalance();
    expect(
      Number(oldContractBalance.output) < Number(newContractBalance.output)
    ).to.be.true;

    // validate that proposalNonce has increased
    let newProposalNonce = await treasuryContract.query.nonce();
    expect(
      Number(newProposalNonce.output) > Number(initialProposalNonce.output)
    ).to.be.true;
  });

  it("Rescue psp22 tokens when amount to rescue is greater than contract balance", async () => {
    let initialProposalNonce = await treasuryContract.query.nonce();

    let oldContractBalance =
      await treasuryContract.query.psp22ContractBalance();
    let oldBobPsp22Balance = await treasuryContract.query.psp22Balance(
      BobSigner.address
    );

    //  Make psp22Contract the owner, and Alice the spender
    let allowedAmount = 500000;
    expect(
      await treasuryContract.tx.setPsp22AllowanceForOwner(
        psp22Contract.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await treasuryContract.query.getPsp22Allowance(
      psp22Contract.address,
      sender.address
    );

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output) === allowedAmount);

    await expect(
      treasuryContract.tx.insertPsp22Balance(psp22Contract.address, 4000)
    ).to.be.fulfilled;

    await expect(
      treasuryContract.tx.insertPsp22Balance(treasuryContract.address, 2000)
    ).to.be.fulfilled;

    // perform token rescue for amount greater than contract balance
    await expect(
      treasuryContract.tx.rescueTokens(
        psp22Contract.address,
        BobSigner.address,
        6000,
        1048
      )
    ).to.be.fulfilled;

    // validate Bob's psp22 balance
    let newBobPsp22Balance = await treasuryContract.query.psp22Balance(
      BobSigner.address
    );
    expect(
      Number(oldBobPsp22Balance.output) < Number(newBobPsp22Balance.output)
    ).to.be.true;

    // validate that proposalNonce has increased
    let newProposalNonce = await treasuryContract.query.nonce();
    expect(
      Number(newProposalNonce.output) > Number(initialProposalNonce.output)
    ).to.be.true;
  });
});
