import { artifacts, network, patract } from "redspot";

const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses } = network;

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