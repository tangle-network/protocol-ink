import { patract, network } from "redspot";
import { flattenDiagnosticMessageText } from "typescript";
import { Abi } from "@polkadot/api-contract"

const { getContractFactory } = patract;
const { createSigner, keyring, api, getSigners } = network;

async function run() {
  await api.isReady;

  // The redspot signer supports passing in an address. If you want to use  substrate uri, you can do it like this:
  // const signer = createSigner(keyring.createFromUri("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"));
  // Or get the configured account from redspot config:
  const signer = (await getSigners())[0]
  // const signer = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"; // Alice address
  // const abi = new Abi(fsExtra.readJson('./artifacts/poseidon.json'), api.registry.getChainProperties())
  // console.log('abis is', abi.info.contract.name)
  // console.log(api.registry.getChainProperties())
  const contractFactory = await getContractFactory("poseidon", signer.address);  // console.log("wasm code is", JSON.stringify(contractFactory.abi.json.V3, null, 2));
  // const inst = api.tx.contracts.instantiateWithCode("100000", "1000", contractFactory.wasm, "", "")
  // console.log("init thinggy", inst)
  // console.log('existential deposit is', api.consts.balances.existentialDeposit.toString())

  // // const balance = await api.query.system.account(signer.address);
  // const contr = await contractFactory.instantiateWithCode("poseidon", {
  //   gas_limit: "1000000"
  // })
  // console.log("contracts", api.query.contracts)

  // console.log("Balance: ", balance.toHuman());

  // The `deploy` method will attempt to deploy a new contract.
  // The `deployed` method will first find out if the same contract already exists based on the parameters.
  // If the contract exists, it will be returned, otherwise a new contract will be created.
  // const mixerInput = {
  //   levels: 3,
  //   deposit_size: 100,
  //   poseidon_contract_hash: '0x0000000000000000000000000000000000000000',
  //   verifier_contract_hash: '0x0000000000000000000000000000000000000000',
  // }

  const contract = await contractFactory.deploy("new", {
    gasLimit: "400000000000",

  });

  console.log("");
  console.log(
    "Deploy successfully. The contract address: ",
    contract.address.toString()
  );

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});

// --> poseidon: 0x9c76b29f1756d3314f35a69ca9e905313157aa50f9eee5f243b828941a85de37
// --> anchor: 0x9e7797cb8f91d4d47a4c82d3ab5b4d13eead7d8536ddc96ee6f44ea5f7e13cb1