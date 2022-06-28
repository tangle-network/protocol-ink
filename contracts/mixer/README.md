#Mixer Contract

The Mixer contract provides functionality for doing deposits and withdrawals from the mixer.

##Dependencies
For the Mixer contract to function properly as it should, The Mixer contract is dependent on these two contracts:
. Poseidon contract: The contract is used for hashing inputs for the Merkle tree.
. Verifier contract: The Verifier contract provides functionality for zero-knowledge verifier management. The contract verifies
Zero knowledge proofs through a chain-extension, this chain-extension is invoked from the Blockchain(Contract-Node) Runtime.

## Setup
The Mixer contract can be instantiated with the following params:
- levels 
- deposit_size 
- version
- poseidon_contract_hash 
- verifier_contract_hash

## Contract Functions
The two major contract functions provided by the Mixer contract are:

- **Deposit**
    
    A Payable contract function for depositing into the mixer. It requires a `commitment` which is the leaf that is inserted
    into the merkle tree.

- **Withdrawal**
    
    A non-payable contract function which is for withdrawing from the mixer. It requires a `WithdrawParams` struct
    
    The `WithdrawParams` struct properties are:
    
    - proof_bytes: The proof to verify for withdrawing from the mixer
    - root: The merkle root of the tree
    - nullifier_hash: You can think of the nullifier hash as a Nonce to stop the replay of withdrawals.
    - recipient: Account address of the recipient
    - relayer: Account address of the relayer
    - fee: Amount paid to the relayer
    - refund: Amount to refund to the recipient



