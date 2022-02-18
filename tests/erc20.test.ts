import { expect } from "chai";
import { artifacts, network, patract } from "redspot";

import { setup } from "../utils";


const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;

// describe("ERC20", () => {
//     after(() => {
//         return api.disconnect();
//     });

//     async function setup() {
//         await api.isReady
//         const signerAddresses = await getAddresses();
//         const Alice = signerAddresses[0];
//         const sender = await getRandomSigner(Alice, "10000000 UNIT");
//         const contractFactory = await getContractFactory("erc20", sender.address);
//         console.log("contractFactory", JSON.stringify(contractFactory, null, 2));
//         const contract = await contractFactory.deploy("new", "1000");
//         const abi = artifacts.readArtifact("erc20");
//         const receiver = await getRandomSigner();

//         return { sender, contractFactory, contract, abi, receiver, Alice };
//     }

//     // it("Assigns initial balance", async () => {
//     //     const { contract, sender } = await setup();
//     //     const result = await contract.query.balanceOf(sender.address);
//     //     expect(result.output).to.equal(1000);
//     // });

//     it("Transfer adds amount to destination account", async () => {
//         const { contract, receiver } = await setup();

//         await expect(() =>
//             contract.tx.transfer(receiver.address, 7)
//         ).to.changeTokenBalance(contract, receiver, 7);

//         await expect(() =>
//             contract.tx.transfer(receiver.address, 7)
//         ).to.changeTokenBalances(contract, [contract.signer, receiver], [-7, 7]);
//     });

//     it("Transfer emits event", async () => {
//         const { contract, sender, receiver } = await setup();

//         await expect(contract.tx.transfer(receiver.address, 7))
//             .to.emit(contract, "Transfer")
//             .withArgs(sender.address, receiver.address, 7);
//     });

//     it("Can not transfer above the amount", async () => {
//         const { contract, receiver } = await setup();

//         await expect(contract.tx.transfer(receiver.address, 1007)).to.not.emit(
//             contract,
//             "Transfer"
//         );
//     });

//     it("Can not transfer from empty account", async () => {
//         const { contract, Alice, sender } = await setup();

//         const emptyAccount = await getRandomSigner(Alice, "10 UNIT");

//         await expect(
//             contract.connect(emptyAccount).tx.transfer(sender.address, 7)
//         ).to.not.emit(contract, "Transfer");
//     });
// });


// to call a "method", you use contract.tx.methodName(args). to get a value, you use contract.query.methodName(args).
describe('mixer', () => {
    after(() => {
        return api.disconnect()
    });

    // i understand that here, i need to create a new mixer. then i can call other methods on it.
    // but my question is: how do i generate these parameters? and what are they? what does levels mean,
    // how further down the tree we wanna go?

    it('Creates a new instance of the mixer', async () => {
        // const poseidonHash = new PoseidonHasher().hash(1, 2, 3)
        const mixerInput = {
            levels: 30,
            deposite_size: 1000,
            poseidon_contract_hash: '0xccef3ab7b72033ca14fa6d6ef82159b998656fba6cf6da0d06f865817b96a8ac',
            verifier_contract_hash: '0x9e4556c4661757959c7afdd546b81cf5546f841e9c104198f2b2f50cb1bf539f',
        }

        const { contract, sender, Alice } = await setup({
            artifact: "mixer",
            factory: "initialize",
        }, mixerInput.levels, mixerInput.deposite_size, mixerInput.poseidon_contract_hash, mixerInput.verifier_contract_hash)
        console.log('Alice is', Alice)
        const result = await contract.query.deposite_size(sender.address)
        console.log('result is', result.output)
        expect(result.output).to.equal(100)
    });
})