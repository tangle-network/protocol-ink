import { expect } from "chai";
import { artifacts, network, patract } from "redspot";
import { CodePromise, ContractPromise, BlueprintPromise, Abi } from "@polkadot/api-contract"
import { u8aToU8a, u8aToHex, u8aConcat, stringToU8a, u8aToString } from '@polkadot/util'
import { readFile } from 'fs/promises'


const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;


// to call a "method", you use contract.tx.methodName(args). to get a value, you use contract.query.methodName(args).
describe('mixer', () => {
    after(() => {
        return api.disconnect()
    });

    async function setup() {
        await api.isReady;
        const signerAddresses = await getAddresses();
        const Alice = signerAddresses[0];
        const sender = await getRandomSigner(Alice, '100 UNIT');

        return { sender, Alice };
    }

    it('Creates a new instance of the mixer', async () => {
        try {
            await api.isReady
            const Alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            // for now, it'd probably be better to get a random signer but since i am trying
            // to get this to work, it shoud be fine (and easier to debug) using just 
            // Alice for now.
            // const signer = await getRandomSigner(Alice, '100 UNIT')
            const signerBalance = await api.query.system.account(Alice)
            // this is correct. checking the substrate-contracts-template frontend, this value is correct
            console.log(`Alice's balance: ${JSON.stringify((signerBalance.toHuman())?.data?.free, null, 2)}`)
            // first, set up things
            const mixerContractAbi = await readFile('./artifacts/mixer.json', { encoding: 'utf8' });
            const abi = new Abi(mixerContractAbi);
            const wasm = await readFile('./target/ink/mixer/mixer.wasm');
            const encodedParams = abi.findConstructor('new').toU8a([30, 100,
                "0xbd7cd0f84cff12a06a99171c18696cd9ccf598abafcd33682172999221874819",
                "0x69c3a4cba7f52ba277baeef8fd8315f30a3631984ecb63f59fe861075a0c88d7"])

            const instance = api.tx.contracts.instantiateWithCode(
                0, // endowment
                100000, // gas fee
                0, // storageLimit
                wasm, // wasm code,
                encodedParams, // encoded params
                null // salt
            )
            // deploy the code now
            // const instantiateWithCode = api.
            console.log('Tx instantiate is', instance.toHuman())

            const tx = instance.withResultTransform(result => result)



            // const { sender, Alice } = await setup();

            // so first we want to create a new code promise. we'll then use the code promise to create a blueprint promise
            // which we'll in-turn use to create the contract promise.
            // however, i am not sure about some of the assumptions in the docs.
            // const mixerCodeInstance = new CodePromise(api, fsExtra.readFileSync('./artifacts/mixer.json', 'utf8'), fsExtra.readFileSync('./target/ink/mixer/mixer.wasm', 'utf8')).

            // // in order to instantiate a contract, we can do `code.tx.<constructor>(args)`
            // // which is what we have here, however this is still not working as i'd expect
            // // because it returns a promise that resolves to Hash when craeted

            // // perhaps then use the Hash to create a new blueprint, that'll give me the address
            // // of the contract on chain and i can then use that to create a new contract promise
            // const instance = await mixerCodeInstance.tx.new({ gasLimit: 100000 }, 30, 100,
            //     "0xbd7cd0f84cff12a06a99171c18696cd9ccf598abafcd33682172999221874819",
            //     "0x69c3a4cba7f52ba277baeef8fd8315f30a3631984ecb63f59fe861075a0c88d7").signAndSend(sender.pair);
            // console.log('Code deployment transaction hash is: ', instance.hash.toHuman())

            // const bluprint = new BlueprintPromise(api, new Abi(mixerCodeInstance.abi.json), instance.hash);
            // const mixerContract = new ContractPromise(api, bluprint.abi, sender.address);
            // const mixerInitialization = await mixerContract.query.initialize(sender.address, { gasLimit: 100000 });
            // // here, i'd assume that the `initialize` method would be called on the contract and return no errors.
            // // but i still get "contractNotFound" error. Possibly i am using the wrong address but tried every address anyway.
            // console.log(mixerInitialization.result.toHuman())

            // // const blueprint = new BlueprintPromise(api, fsExtra.readFileSync('./artifacts/mixer.json', 'utf8'), instance.hash);
            // console.log('Blueprint instance tx hash is', blueprint.codeHash.toString())
            // const contract = new ContractPromise(api, new Abi(blueprint.abi.json), blueprint.codeHash.toString());
            // console.log(instance.hash.toHuman())
            // const ppt = await instance.dryRun(sender.address)
            // the docs specify that the blueprint promise takes argument "codeHash" as either hex string, Hash or Uint8Array
            // however now it seems only string and Hash are supported.
            // const mixerBlueprintInstance = new BlueprintPromise(api, mixerCodeInstance.abi, mixerCodeInstance.code.toString());

            // // now let me try to create a contract from this. we sign the tx using the sender address.
            // // now in the docs, this should resolve to a promise of `result` (which contrains the status and the contract itself)
            // // but here it resolves to a `Hash`, so that means the hash should be the on-chain hash of the contract.
            // const mixerContractInstance = await mixerBlueprintInstance.tx.new({
            //     gasLimit: 1000000
            // }, 30, 100,
            //     "0xbd7cd0f84cff12a06a99171c18696cd9ccf598abafcd33682172999221874819",
            //     "0x69c3a4cba7f52ba277baeef8fd8315f30a3631984ecb63f59fe861075a0c88d7").signAndSend(sender.address)
            // console.log('Mixer contract instantiation hash is', mixerContractInstance.hash)

            // const { } = mixerBlueprintInstance.query


            // now create a new contract instance
            // const mixerContract = new ContractPromise(api, mixerCodeInstance.abi, sender.address)
            // // according to the docs, if we passed a wrong code hash in the blueprint promise, it will throw an error.
            // console.log('Mixer contract', mixerContract)



            // const bp = await code.createBlueprint(sender.address, 'mixer');
            // const pxx = code.tx.new({ gasLimit: 100000 }, 30, 100,
            //     "0xbd7cd0f84cff12a06a99171c18696cd9ccf598abafcd33682172999221874819",
            //     "0x69c3a4cba7f52ba277baeef8fd8315f30a3631984ecb63f59fe861075a0c88d7").signAndSend(sender.address)


            // const contr = new BlueprintPromise(api, new Abi(code.abi.json), (await pxx).hash)
            // console.log('new blueprint is: ', contr.codeHash.toString())
            // console.log(`code is`, code.abi.metadata.hash.toHex())
            // // Deploy the WASM, retrieve a Blueprint
            // const blueprint = new BlueprintPromise(api, new Abi(code.abi.json), sender.address);
            // // console.log(`Blueprint is`, blueprint.)
            // console.log('blueprint on-chain hash is', blueprint.codeHash.toString())
            // const mixerContract = new ContractPromise(api, new Abi(code.abi.json), blueprint.codeHash.toJSON);
            // console.log(`contract is`, await (await mixerContract.query.initialize(sender.address, {})).result.toHuman())
            // const inst = await mixerContract.query.new(blueprint.codeHash.toString(), { gasLimit: 1000000 }, 30, 100,
            //     "0xbd7cd0f84cff12a06a99171c18696cd9ccf598abafcd33682172999221874819",
            //     "0x69c3a4cba7f52ba277baeef8fd8315f30a3631984ecb63f59fe861075a0c88d7");
            // console.log('another instance is', inst.result)
            // console.log('mixerContract is', mixerContract.query)
            // const nw = await mixerContract.tx.new({ gasLimit: 1000000 }, 30, 100,
            //     "0xbd7cd0f84cff12a06a99171c18696cd9ccf598abafcd33682172999221874819",
            //     "0x69c3a4cba7f52ba277baeef8fd8315f30a3631984ecb63f59fe861075a0c88d7").signAndSend(sender.pair);
            // console.log('mixer contract is', nw.hash.toHex())

            // const mxx = await mixerContract.query.initialize(mixerContract.address, { gasLimit: 1000000 });
            // console.log(`mixer initialization is`, mxx.result.toHuman())


            // const signedMixerInstantiation = await blueprint.tx.new({ gasLimit: 1000000, }, 30, 100,
            //     "0xbd7cd0f84cff12a06a99171c18696cd9ccf598abafcd33682172999221874819",
            //     "0x69c3a4cba7f52ba277baeef8fd8315f30a3631984ecb63f59fe861075a0c88d7").signAndSend(sender.address, {
            //         signer: sender
            //     })

            // // console.log(signedMixerInstantiation)


            // const mixerContract = new ContractPromise(api, new Abi(code.abi.json), signedMixerInstantiation.h);
            // console.log('mixerContract is', mixerContract.address.toString())

            // // console.log(`Initialize function fetched from ABI for contract: `, mixerContract.abi.findMessage('initialize').returnType)
            // const init = await mixerContract.api.rpc.contracts.call({
            //     origin: sender.address,
            //     gasLimit: 1000000
            // });
            // console.log('exectution response is: ', init.result.toHuman())
            // const mxx = await mixerContract.api.tx.initialize()
            // const mox = await mixerContract.query.initialize(sender.address, { gasLimit: 1000000, storageDepositLimit: 10000 })

            // if (mox.result.isOk) {
            //     console.log('successfully initialized the mixer contract', mox.result.toHuman())
            // } else {
            //     console.log(`Error initializing the mixer contract`, mox.result.asErr.toHuman())
            // }
            // console.log(`mixer contract is`, mixerContract.address.toHuman())
            // const mixerInit = await mixerContract.tx.initialize({ gasLimit: 1000000 }).signAndSend(sender.address)
            // console.log(`mixer init is`, mixerInit.toHuman())

        } catch (error) {
            console.log('error creating an instance of mixer', error)
        }
    });
}) 