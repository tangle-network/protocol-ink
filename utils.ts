import { artifacts, network, patract } from "redspot";

const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;

export async function setup({ artifact, factory }, ...factoryArgs) {
    await api.isReady
    const signerAddresses = await getAddresses();
    const Alice = signerAddresses[0];
    console.log("Alice", Alice);
    const sender = await getRandomSigner(Alice, "10000 UNIT");
    console.log('addreses gotten', JSON.stringify(sender, null, 2))
    console.log('contracts are: ', JSON.stringify(api.tx.contracts, null, 2))
    const contractFactory = await getContractFactory(artifact, sender.address);
    const dropOut = await api.query.system.account(sender.address)
    // console.log('balance of the accouint is', dropOut.toHuman())
    console.log(`contractFactory`, contractFactory);
    const contract = await contractFactory.deploy(factory, ...factoryArgs);
    const abi = artifacts.readArtifact(artifact);
    const receiver = await getRandomSigner();

    return { sender, contractFactory, contract, abi, receiver, Alice };
}

export const chainsConfig = {
    id: "WebbDevelopment",
    group: 'webb',
    tag: 'dev',
    evmId: undefined,
    // logo: WEBBLogo,x
    url: 'ws://127.0.0.1:9944',
    name: 'Webb Development',
    currencies: "WEBB",
    nativeCurrencyId: "WEBB",
}