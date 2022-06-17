import { expect } from "chai";
import { artifacts, network, patract } from "redspot";
import BN from "bn.js";
import { all } from "@polkadot/api-derive/balances";
import { hexToU8a } from "@polkadot/util";
import {ChildProcess, spawn} from 'child_process';

const { getContractFactory, getRandomSigner } = patract;
const { api, getAddresses, getSigners } = network;
const { exec } = require("child_process");

export function normalizeBalance(balance: number): BN {
  return new BN(balance).mul(
    new BN(10).pow(new BN(api.registry.chainDecimals[0]))
  );
}

describe("token-wrapper", () => {
  let tokenWrapperContract: any;
  let tokenWrapperContractFactory: any;
  let sender: any;
  let BobSigner: any;
  let CharlieSigner: any;
  let FerdieSigner: any;
  let EveSigner: any;
  let DaveSigner: any;
  let tokenName: any;
  let contractProposalNonce: any;
  let contractGovernor: any;
  let isNativeAllowed: any;
  let wrappingLimit: any;
  let feeRecipient: any;
  let feePercentage: any;
  let psp22Contract: any;
  let ls;
  after(() => {
    //if (ls) ls.kill('SIGINT');
    return api.disconnect();
  });


  before(async () => {
    console.log("SPAWNING");
    const startArgs: string[] = [];
    startArgs.push("--tmp -lruntime=debug -linfo")
    ls = spawn( './artifacts/substrate-contracts-node-linux/substrate-contracts-node',
    []);

    ls.stdout.on('data', (data) => {
      console.log(`printing out data`);
      console.log(`stdout: ${data}`);
    });

    ls.stderr.on('data', (data) => {
      console.log(`printing out error`);
      console.error(`stderr: ${data}`);
    });

    /*exec('php main.php', function (error, stdOut, stdErr) {
      console.log(`stdout: ${stdOut}`);
      console.log(`stdout: ${stdErr}`);
      console.log(`stdout: ${error}`);
    });*/

    /*await exec('/Users/Damilare/Documents/self-paced/Substrate/contract_node/substrate-contracts-node/target/release/substrate-contracts-node',function (err,stdout,stderr) {
      if (err) {
        console.log("\n"+stderr);
      } else {
        console.log(stdout);
      }
    });*/

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
      tokenName,
      contractProposalNonce,
      isNativeAllowed,
      wrappingLimit,
      feeRecipient,
      feePercentage,
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
      governorBalance,
    } = await tokenWrapperContractInitParams(
      sender,
      BobSigner,
      CharlieSigner,
      DaveSigner
    );

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
      tokenName,
      contractProposalNonce,
      isNativeAllowed,
      wrappingLimit,
      feeRecipient,
      feePercentage,
      psp22Contract,
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

  it.only("Add token address", async () => {
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output).to.equal(true);

    // validate that proposalNonce has increased
    let newProposalNonce = await tokenWrapperContract.query.nonce();
    expect(newProposalNonce.output === contractProposalNonce + 1);
  });

  it("Remove token address", async () => {
    // first add a token address
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output === true);

    // validate that proposalNonce has increased
    let newProposalNonce = await tokenWrapperContract.query.nonce();
    expect(newProposalNonce.output > contractProposalNonce + 1);

    // increase nonce
    let proposalNonce = Number(newProposalNonce.output) + 1;

    // now remove token address
    expect(
      await tokenWrapperContract.tx.removeTokenAddress(
        psp22Contract.address,
        proposalNonce
      )
    ).to.be.ok;

    // validate that address has been removed successfully
    let isValidAddressAgain =
      await tokenWrapperContract.query.isValidTokenAddress(
        psp22Contract.address
      );
    expect(isValidAddressAgain.output === false);

    // validate that proposalNonce has increased
    let newProposalNonceAgain = await tokenWrapperContract.query.nonce();
    expect(newProposalNonceAgain.output === proposalNonce);
  });

  it("Update config", async () => {
    let newGovernor = BobSigner.address;
    let newIsNativeAllowed = false;
    let newWrappingLimit = 1000;
    let newFeePercentage = 2;
    let newFeeRecipient = FerdieSigner.address;

    // update config with new states
    expect(
      await tokenWrapperContract.tx.updateConfig(
        newGovernor,
        newIsNativeAllowed,
        newWrappingLimit,
        newFeePercentage,
        newFeeRecipient
      )
    ).to.be.ok;

    // validate that new governor is not same as old governor
    let newGovernorFromStorage = await tokenWrapperContract.query.governor();
    expect(newGovernorFromStorage.output === newGovernor);
    expect(newGovernorFromStorage.output === contractGovernor);

    // validate that isNativeAllowed has changed
    let newIsNativeAllowedFromStorage =
      await tokenWrapperContract.query.isNativeAllowed();
    expect(newIsNativeAllowedFromStorage.output === newIsNativeAllowed);
    expect(newIsNativeAllowedFromStorage.output != isNativeAllowed);

    // validate that isWrappingLimit has changed
    let newWrappingLimitFromStorage =
      await tokenWrapperContract.query.wrappingLimit();
    expect(Number(newWrappingLimitFromStorage.output === newWrappingLimit));
    expect(Number(newWrappingLimitFromStorage.output != wrappingLimit));

    // validate that feePercentage has changed
    let newFeePercentageFromStorage =
      await tokenWrapperContract.query.feePercentage();
    expect(Number(newFeePercentageFromStorage.output === newFeePercentage));
    expect(Number(newFeePercentageFromStorage.output != feePercentage));

    // validate that feeRecipient has changed
    let newFeeRecipientFromStorage =
      await tokenWrapperContract.query.feeRecipient();
    expect(newFeeRecipientFromStorage.output === newFeeRecipient);
    expect(newFeeRecipientFromStorage.output != feeRecipient);
  });

  it("Test native wrapping functionality", async () => {
    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    let initialContractBalance =
      await tokenWrapperContract.query.nativeContractBalance();

    expect(Number(initialSenderWrappedBalance.output) === 0);
    expect(Number(initialContractBalance.output) === 0);

    let wrapFunction = await tokenWrapperContract.tx.wrap(null, 0, {
      value: 1500,
    });

    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for sender
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    // to validate that native funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.nativeContractBalance();

    expect(
      Number(senderWrappedBalanceAfter.output) >
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;
  });

  it("Test psp22 wrapping functionality", async () => {
    // first add token address
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output).to.equal(true);

    // secondly set psp22 allowance
    let allowedAmount = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22Allowance(
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(
      sender.address,
      sender.address
    );

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output)).to.be.equal(allowedAmount);

    let newFeeRecipient = EveSigner.address;

    // update config with new states
    let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(
      0,
      0,
      1_000_000_000_01,
      1,
      newFeeRecipient
    );
    expect(updateConfigFunction).to.be.ok;

    // validate that feeRecipient has changed
    let newFeeRecipientFromStorage =
      await tokenWrapperContract.query.feeRecipient();
    expect(newFeeRecipientFromStorage.output === EveSigner.address);
    expect(newFeeRecipientFromStorage.output != feeRecipient);

    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    let initialContractBalance =
      await tokenWrapperContract.query.psp22ContractBalance();
    let initialFeeRecipientBalance =
      await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

    // insert balance for Alice(sender)
    let insertBalanceFunction =
      await tokenWrapperContract.tx.insertPsp22Balance(
        sender.address,
        8_000_000
      );
    expect(insertBalanceFunction).to.be.ok;

    // now do wrapping
    let wrapFunction = await tokenWrapperContract.tx.wrap(
      psp22Contract.address,
      1
    );
    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for sender
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    // to validate that psp22 token funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.psp22ContractBalance();
    // to validate that fee has been transferred to the fee recipient
    let feeRecipientBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(feeRecipient);

    expect(
      Number(senderWrappedBalanceAfter.output) !=
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;
    expect(
      Number(feeRecipientBalanceAfter.output) >
        Number(initialFeeRecipientBalance.output)
    ).to.be.true;
  });

  it("Test native wrapping for functionality", async () => {
    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    let initialContractBalance =
      await tokenWrapperContract.query.nativeContractBalance();

    let wrapFunction = await tokenWrapperContract.tx.wrapFor(
      null,
      FerdieSigner.address,
      0,
      { value: 1000 }
    );

    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for sender, in this case Ferdie
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    // to validate that native funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.nativeContractBalance();

    expect(
      Number(senderWrappedBalanceAfter.output) >
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;
  });

  it("Test psp22 wrapping for functionality", async () => {
    // first add token address
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output === true);

    // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
    let allowedAmount = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22AllowanceForOwner(
        FerdieSigner.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(
      FerdieSigner.address,
      sender.address
    );

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output) === allowedAmount);

    // update config with new states
    let newFeeRecipient = EveSigner.address;
    let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(
      0,
      0,
      9_000_000_000,
      1,
      newFeeRecipient
    );
    expect(updateConfigFunction).to.be.ok;

    // validate that feeRecipient has changed
    let newFeeRecipientFromStorage =
      await tokenWrapperContract.query.feeRecipient();
    expect(newFeeRecipientFromStorage.output === EveSigner.address);
    expect(newFeeRecipientFromStorage.output != feeRecipient);

    // insert balance for Alice(sender)
    let insertBalanceFunction =
      await tokenWrapperContract.tx.insertPsp22Balance(
        sender.address,
        8_000_000
      );
    expect(insertBalanceFunction).to.be.ok;

    // Fund Ferdies account
    expect(
      await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)
    ).to.be.ok;

    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    let initialContractBalance =
      await tokenWrapperContract.query.psp22ContractBalance();
    let initialFeeRecipientBalance =
      await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

    // now do wrapping for Ferdie
    let wrapFunction = await tokenWrapperContract.tx.wrapFor(
      psp22Contract.address,
      FerdieSigner.address,
      10
    );
    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for Ferdie
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    // to validate that psp22 token funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.psp22ContractBalance();
    // to validate that fee has been transferred to the fee recipient
    let feeRecipientBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(feeRecipient);

    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;
    expect(
      Number(feeRecipientBalanceAfter.output) >
        Number(initialFeeRecipientBalance.output)
    ).to.be.true;
  });

  it("Test native wrapping for and send to functionality", async () => {
    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(EveSigner.address);
    let initialContractBalance =
      await tokenWrapperContract.query.nativeContractBalance();

    let wrapFunction = await tokenWrapperContract.tx.wrapForAndSendTo(
      null,
      FerdieSigner.address,
      0,
      EveSigner.address,
      { value: 1000 }
    );

    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for the recipient specified, in this case Eve
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(EveSigner.address);
    // to validate that native funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.nativeContractBalance();

    expect(
      Number(senderWrappedBalanceAfter.output) >
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;
  });

  it("Test psp22 wrapping for and send to functionality", async () => {
    // first add token address
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output === true);

    // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
    let allowedAmount = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22AllowanceForOwner(
        FerdieSigner.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(
      FerdieSigner.address,
      sender.address
    );

    // update config with new states
    let newFeeRecipient = BobSigner.address;
    let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(
      0,
      0,
      9_000_000_000,
      1,
      newFeeRecipient
    );
    expect(updateConfigFunction).to.be.ok;

    // validate that feeRecipient has changed
    let newFeeRecipientFromStorage =
      await tokenWrapperContract.query.feeRecipient();
    expect(newFeeRecipientFromStorage.output === BobSigner.address);
    expect(newFeeRecipientFromStorage.output != feeRecipient);

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output) === allowedAmount);

    // insert balance for Alice(sender)
    let insertBalanceFunction =
      await tokenWrapperContract.tx.insertPsp22Balance(
        sender.address,
        8_000_000
      );
    expect(insertBalanceFunction).to.be.ok;

    // Fund Ferdies account
    expect(
      await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)
    ).to.be.ok;

    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(EveSigner.address);
    let initialContractBalance =
      await tokenWrapperContract.query.psp22ContractBalance();
    let initialFeeRecipientBalance =
      await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

    // now do wrapping for Ferdie
    let wrapFunction = await tokenWrapperContract.tx.wrapForAndSendTo(
      psp22Contract.address,
      FerdieSigner.address,
      1000,
      EveSigner.address
    );
    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for Ferdie
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(EveSigner.address);
    // to validate that psp22 token funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.psp22ContractBalance();
    // to validate that fee has been transferred to the fee recipient
    let feeRecipientBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

    expect(
      Number(feeRecipientBalanceAfter.output) >
        Number(initialFeeRecipientBalance.output)
    ).to.be.true;
    expect(
      Number(senderWrappedBalanceAfter.output) >
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;
  });

  it("Test native unwrap functionality", async () => {
    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    let initialContractBalance =
      await tokenWrapperContract.query.nativeContractBalance();

    // first do wrapping first
    let wrapFunction = await tokenWrapperContract.tx.wrap(null, 0, {
      value: 1000,
    });

    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for sender
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    // to validate that native funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.nativeContractBalance();

    expect(
      Number(senderWrappedBalanceAfter.output) >
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;

    let unwrapFunction = await tokenWrapperContract.tx.unwrap(null, 8, {
      value: 1000,
    });

    expect(unwrapFunction).to.be.ok;

    let senderBurntBalance = await tokenWrapperContract.query.psp22Balance(
      sender.address
    );

    // validate that balance has reduced for sender
    expect(Number(senderWrappedBalanceAfter.output) > senderBurntBalance.output)
      .to.be.true;
  });

  it("Test native unwrapping for functionality", async () => {
    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    let initialContractBalance =
      await tokenWrapperContract.query.nativeContractBalance();

    let wrapFunction = await tokenWrapperContract.tx.wrapFor(
      null,
      FerdieSigner.address,
      0,
      { value: 1000 }
    );

    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for sender, in this case Ferdie
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    // to validate that native funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.nativeContractBalance();

    expect(
      Number(senderWrappedBalanceAfter.output) >
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;

    let unwrapFunction = await tokenWrapperContract.tx.unwrapFor(
      null,
      100,
      FerdieSigner.address,
      { value: 1000 }
    );

    expect(unwrapFunction).to.be.ok;

    // to validate that psp22 token has been burnt for sender, in this case Ferdie
    let senderBurntBalance = await tokenWrapperContract.query.psp22Balance(
      FerdieSigner.address
    );

    expect(
      Number(senderWrappedBalanceAfter.output) >
        Number(senderBurntBalance.output)
    ).to.be.true;
  });

  it("Test native wrapping for and send to functionality", async () => {
    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(EveSigner.address);
    let initialContractBalance =
      await tokenWrapperContract.query.nativeContractBalance();

    let wrapFunction = await tokenWrapperContract.tx.wrapForAndSendTo(
      null,
      FerdieSigner.address,
      0,
      EveSigner.address,
      { value: 1000 }
    );

    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for the recipient specified, in this case Eve
    let eveWrappedBalanceAfter = await tokenWrapperContract.query.psp22Balance(
      EveSigner.address
    );
    // to validate that native funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.nativeContractBalance();

    expect(
      Number(eveWrappedBalanceAfter.output) >
        Number(initialSenderWrappedBalance.output)
    ).to.be.true;
    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;

    // insert balance for Alice(sender)
    let insertBalanceFunction =
      await tokenWrapperContract.tx.insertPsp22Balance(
        sender.address,
        8_000_000
      );
    expect(insertBalanceFunction).to.be.ok;

    let senderBalance = await tokenWrapperContract.query.psp22Balance(
      sender.address
    );

    let unwrapFunction = await tokenWrapperContract.tx.unwrapAndSendTo(
      null,
      2,
      sender.address,
      { value: 1000 }
    );

    expect(unwrapFunction).to.be.ok;

    let senderBurntBalance = await tokenWrapperContract.query.psp22Balance(
      sender.address
    );

    // validate that balance has reduced for sender
    expect(Number(senderBalance.output) > Number(senderBurntBalance.output)).to
      .be.true;
  });

  it("Test psp22 unwrap functionality", async () => {
    // first add token address
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output === true);

    // secondly set psp22 allowance
    let allowedAmount = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22Allowance(
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(
      sender.address,
      sender.address
    );

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output) === allowedAmount);

    // update config with new states
    let newFeeRecipient = EveSigner.address;
    let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(
      0,
      0,
      9_000_000_000,
      1,
      newFeeRecipient
    );
    expect(updateConfigFunction).to.be.ok;

    // validate that feeRecipient has changed
    let newFeeRecipientFromStorage =
      await tokenWrapperContract.query.feeRecipient();
    expect(newFeeRecipientFromStorage.output === EveSigner.address);

    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    let initialContractBalance =
      await tokenWrapperContract.query.psp22ContractBalance();
    let initialFeeRecipientBalance =
      await tokenWrapperContract.query.psp22Balance(newFeeRecipient);

    // insert balance for Alice(sender)
    let insertBalanceFunction =
      await tokenWrapperContract.tx.insertPsp22Balance(
        sender.address,
        8_000_000
      );
    expect(insertBalanceFunction).to.be.ok;

    // now do wrapping
    let wrapFunction = await tokenWrapperContract.tx.wrap(
      psp22Contract.address,
      10000
    );
    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for sender
    let senderWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(sender.address);
    // to validate that psp22 token funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.psp22ContractBalance();
    // to validate that fee has been transferred to the fee recipient
    let feeRecipientBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(feeRecipient);

    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    ).to.be.true;
    expect(
      Number(feeRecipientBalanceAfter.output) >
        Number(initialFeeRecipientBalance.output)
    ).to.be.true;

    // secondly set psp22 allowance
    expect(
      await tokenWrapperContract.tx.setPsp22AllowanceForOwner(
        tokenWrapperContract.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSetForContract =
      await tokenWrapperContract.query.getPsp22Allowance(
        tokenWrapperContract.address,
        sender.address
      );

    // validate that psp22 allowance was set
    expect(Number(allowanceSetForContract.output) === allowanceSetForContract);

    // validate that psp22 allowance was set
    expect(Number(allowanceSetForContract.output) === allowedAmount);
    // now do unwrapping
    let unwrapFunction = await tokenWrapperContract.tx.unwrap(
      psp22Contract.address,
      1000
    );
    expect(unwrapFunction).to.be.ok;

    let senderBurntBalanceAfter = await tokenWrapperContract.query.psp22Balance(
      sender.address
    );

    // to validate that psp22 token funds was transferred from the contract
    let contractBalanceAfterUnwrap =
      await tokenWrapperContract.query.psp22ContractBalance();

    expect(
      Number(contractBalanceAfter.output) >
        Number(contractBalanceAfterUnwrap.output)
    ).to.be.true;
  });

  it("Test psp22 unwrap for functionality", async () => {
    // first add token address
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;

    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output == true);

    // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
    let allowedAmount = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22AllowanceForOwner(
        FerdieSigner.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(
      FerdieSigner.address,
      sender.address
    );

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output)).to.be.equal(allowedAmount);

    // insert balance for Alice(sender)
    let insertBalanceFunction =
      await tokenWrapperContract.tx.insertPsp22Balance(
        sender.address,
        8_000_000
      );
    expect(insertBalanceFunction).to.be.ok;

    // Fund Ferdies account
    expect(
      await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)
    ).to.be.ok;

    let initialSenderWrappedBalance =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    let initialContractBalance =
      await tokenWrapperContract.query.psp22ContractBalance();
    let initialFeeRecipientBalance =
      await tokenWrapperContract.query.psp22Balance(feeRecipient);

    expect(Number(initialSenderWrappedBalance.output) != 0);
    expect(Number(initialContractBalance.output) == 0);

    // update config with new states
    let newFeeRecipient = EveSigner.address;
    let updateConfigFunction = await tokenWrapperContract.tx.updateConfig(
      0,
      0,
      9_000_000_000,
      1,
      newFeeRecipient
    );
    expect(updateConfigFunction).to.be.ok;

    // validate that feeRecipient has changed
    let newFeeRecipientFromStorage =
      await tokenWrapperContract.query.feeRecipient();
    expect(newFeeRecipientFromStorage.output === EveSigner.address);

    // now do wrapping for Ferdie
    let wrapFunction = await tokenWrapperContract.tx.wrapFor(
      psp22Contract.address,
      FerdieSigner.address,
      10
    );
    expect(wrapFunction).to.be.ok;

    // to validate that psp22 token has been minted for Ferdie
    let FerdieWrappedBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(FerdieSigner.address);
    // to validate that psp22 token funds was transferred to the contract
    let contractBalanceAfter =
      await tokenWrapperContract.query.psp22ContractBalance();
    // to validate that fee has been transferred to the fee recipient
    let feeRecipientBalanceAfter =
      await tokenWrapperContract.query.psp22Balance(feeRecipient);

    expect(Number(FerdieWrappedBalanceAfter.output) != 0);
    expect(Number(contractBalanceAfter.output) != 0);
    expect(Number(feeRecipientBalanceAfter.output) != 0);

    expect(
      Number(contractBalanceAfter.output) >
        Number(initialContractBalance.output)
    );

    // secondly set psp22 allowance
    let allowedAmountForContract = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22AllowanceForOwner(
        tokenWrapperContract.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSetForContract =
      await tokenWrapperContract.query.getPsp22Allowance(
        tokenWrapperContract.address,
        sender.address
      );

    // validate that psp22 allowance for contract was set
    expect(Number(allowanceSetForContract.output) == allowedAmountForContract);

    // now do unwrapping
    let unwrapFunction = await tokenWrapperContract.tx.unwrapFor(
      psp22Contract.address,
      5,
      FerdieSigner.address
    );
    expect(unwrapFunction).to.be.ok;

    let FerdieBurntBalanceAfter = await tokenWrapperContract.query.psp22Balance(
      sender.address
    );

    // to validate that psp22 token funds was transferred from the contract
    let contractBalanceAfterUnwrap =
      await tokenWrapperContract.query.psp22ContractBalance();
    expect(
      Number(contractBalanceAfter.output) >
        Number(contractBalanceAfterUnwrap.output)
    );
    expect(
      Number(FerdieWrappedBalanceAfter.output) >
        Number(FerdieBurntBalanceAfter.output)
    );
  });

  it("Test psp22 unwrapping and send to functionality", async () => {
    // first add token address
    expect(
      await tokenWrapperContract.tx.addTokenAddress(
        psp22Contract.address,
        contractProposalNonce + 1
      )
    ).to.be.ok;
    // validate that address has been added successfully
    let isValidAddress = await tokenWrapperContract.query.isValidTokenAddress(
      psp22Contract.address
    );
    expect(isValidAddress.output === true);

    // secondly set psp22 allowance. Make Ferdie the owner, and Alice the spender
    let allowedAmount = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22AllowanceForOwner(
        FerdieSigner.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSet = await tokenWrapperContract.query.getPsp22Allowance(
      FerdieSigner.address,
      sender.address
    );

    // validate that psp22 allowance was set
    expect(Number(allowanceSet.output) === allowedAmount);

    // insert balance for Alice(sender)

    expect(
      await tokenWrapperContract.tx.insertPsp22Balance(
        sender.address,
        8_000_000
      )
    ).to.be.ok;

    // insert balance for Contract
    expect(
      await tokenWrapperContract.tx.insertPsp22Balance(
        tokenWrapperContract.address,
        8_000_000
      )
    ).to.be.ok;

    // Fund Ferdies account
    expect(
      await tokenWrapperContract.tx.transferPsp22(FerdieSigner.address, 100000)
    ).to.be.ok;

    let contractBalanceAfter =
      await tokenWrapperContract.query.psp22ContractBalance();

    // set psp22 allowance for contract
    let allowedAmountForContract = 500000;
    expect(
      await tokenWrapperContract.tx.setPsp22AllowanceForOwner(
        tokenWrapperContract.address,
        sender.address,
        allowedAmount
      )
    ).to.be.ok;

    let allowanceSetForContract =
      await tokenWrapperContract.query.getPsp22Allowance(
        tokenWrapperContract.address,
        sender.address
      );

    // to validate that psp22 token has been burnt for Eve
    let SenderInitialPsp22Balance =
      await tokenWrapperContract.query.psp22Balance(sender.address);

    // validate that psp22 allowance for contract was set
    expect(Number(allowanceSetForContract.output) === allowedAmountForContract);

    // now do unwrapping
    let unwrapFunction = await tokenWrapperContract.tx.unwrapAndSendTo(
      psp22Contract.address,
      1000,
      FerdieSigner.address
    );
    expect(unwrapFunction).to.be.ok;

    // to validate that psp22 token has been burnt for Sender
    let SenderBurntPsp22Balance = await tokenWrapperContract.query.psp22Balance(
      sender.address
    );

    // to validate that psp22 token funds was transferred from the contract
    let contractBalanceAfterUnwrap =
      await tokenWrapperContract.query.psp22ContractBalance();

    expect(
      Number(contractBalanceAfter.output) >
        Number(contractBalanceAfterUnwrap.output)
    );
    expect(
      Number(SenderInitialPsp22Balance.output) >
        Number(SenderBurntPsp22Balance.output)
    );
  });
});
