import { expect } from "chai";
import { artifacts, network, patract } from "redspot";
import BN from "bn.js";

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;

export function normalizeBalance(balance: number): BN {
  return new BN(balance).mul(
    new BN(10).pow(new BN(api.registry.chainDecimals[0]))
  );
}

describe("token-wrapper-negative-tests", () => {
  let tokenWrapperContract: any;
  let tokenWrapperContractFactory: any;
  let sender: any;
  let BobSigner: any;
  let CharlieSigner: any;
  let FerdieSigner: any;
  let EveSigner: any;
  let DaveSigner: any;
  let psp22Contract: any;
  after(() => {
    return api.disconnect();
  });

  before(async () => {
    await api.isReady;
  });

  beforeEach(async () => {
    ({
      tokenWrapperContract,
      tokenWrapperContractFactory,
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
    } = tokenWrapperContractInitParams(sender, BobSigner, CharlieSigner);

    // token wrapper instantiation
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

    // create a psp22 token contract to use as token address
    const psp22ContractFactory = await getContractFactory(
      "psp22_token",
      BobSigner.address
    );

    const psp22Contract = await psp22ContractFactory.deploy(
      "new",
      10000,
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
      tokenWrapperContractFactory,
      tokenWrapperContract,
      psp22Contract,
    };
  }

  function tokenWrapperContractInitParams(
    sender: any,
    BobSigner: any,
    CharlieSigner: any
  ) {
    let tokenName = "Webb";
    let tokenSymbol = "Webb";
    let decimal = 4;
    let contractGovernor = sender.address;
    let feeRecipient = sender.address;
    let feePercentage = 1;
    let isNativeAllowed = true;
    let wrappingLimit = 10;
    let contractProposalNonce = 2;
    let totalSupply = 1_000_000_000_000_000;

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
      totalSupply,
    };
  }

  it("Add token address with smaller nonce than what exists should fail", async () => {
    const { tokenName, contractProposalNonce } = tokenWrapperContractInitParams(
      sender,
      BobSigner,
      CharlieSigner
    );

      await expect(
        tokenWrapperContract.tx.addTokenAddress(psp22Contract.address, 1)
      ).to.not.be.fulfilled;

    // validate that address has not been added since an error occured
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output).to.equal(false);

    // validate that proposalNonce has increased
    let newProposalNonce = await tokenWrapperContract.query.nonce();
    expect(newProposalNonce.output).to.not.equal(contractProposalNonce + 1);
  });

  it("Add token address with same nonce as what exists should fail", async () => {
    const { tokenName, contractProposalNonce } = tokenWrapperContractInitParams(
      sender,
      BobSigner,
      CharlieSigner
    );

      await expect(
        tokenWrapperContract.tx.addTokenAddress(
          psp22Contract.address,
          contractProposalNonce
        )
      ).to.not.be.fulfilled;


    // validate that address has not been added since an error occured
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output).to.equal(false);

    // validate that proposalNonce has increased
    let newProposalNonce = await tokenWrapperContract.query.nonce();
    expect(newProposalNonce.output).to.not.equal(contractProposalNonce + 1);
  });

  it("Removing token address with a smaller nonce than what exists should fail", async () => {
    const { contractProposalNonce } = tokenWrapperContractInitParams(
      sender,
      BobSigner,
      CharlieSigner
    );

    // first add a token address
    let addTokenFunction = await tokenWrapperContract.tx.addTokenAddress(
      psp22Contract.address,
      contractProposalNonce + 1
    );
    expect(addTokenFunction).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output).to.equal(true);

    // validate that proposalNonce has increased
    let newProposalNonce = await tokenWrapperContract.query.nonce();
    expect(newProposalNonce.output).to.be.equal(contractProposalNonce + 1);


      await expect(
        tokenWrapperContract.tx.removeTokenAddress(psp22Contract.address, 1)
      ).to.to.not.be.fulfilled;

    // validate that address has not been removed successfully, so address will still be valid
    let isValidAddressAgain =
      await tokenWrapperContract.query.isValidTokenAddress(
        psp22Contract.address
      );
    expect(isValidAddressAgain.output).to.equal(true);

    // validate that proposalNonce has not increased
    let newProposalNonceAgain = await tokenWrapperContract.query.nonce();
    expect(
      Number(newProposalNonceAgain.output) == Number(newProposalNonce.output)
    ).to.be.true;
  });

  it("Removing a token address with the same nonce that exists should fail", async () => {
    const { contractProposalNonce } = tokenWrapperContractInitParams(
      sender,
      BobSigner,
      CharlieSigner
    );

    // first add a token address
    let addTokenFunction = await tokenWrapperContract.tx.addTokenAddress(
      psp22Contract.address,
      contractProposalNonce + 1
    );
    expect(addTokenFunction).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output).to.equal(true);

    // validate that proposalNonce has increased
    let newProposalNonce = await tokenWrapperContract.query.nonce();
    expect(newProposalNonce.output).to.be.equal(contractProposalNonce + 1);

    // increase nonce
    // @ts-ignore
    let proposalNonce = Number(newProposalNonce.output) + 1;

    console.log(`proposalNonce is ${proposalNonce}`);


      await expect(
        tokenWrapperContract.tx.removeTokenAddress(
          psp22Contract.address,
          Number(newProposalNonce.output)
        )
      ).to.not.be.fulfilled;

    // validate that address has not been removed successfully, so address will still be valid
    let isValidAddressAgain =
      await tokenWrapperContract.query.isValidTokenAddress(
        psp22Contract.address
      );
    expect(isValidAddressAgain.output).to.equal(true);
    console.log(`new proposalNonce is ${isValidAddressAgain.output}`);

    // validate that proposalNonce has not increased
    let newProposalNonceAgain = await tokenWrapperContract.query.nonce();
    expect(
      Number(newProposalNonceAgain.output) == Number(newProposalNonce.output)
    ).to.be.true;
  });
});
