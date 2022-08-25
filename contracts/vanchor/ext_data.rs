use ark_bn254::Fr as Bn254Fr;
use ark_ff::{BigInteger, PrimeField};
use ethabi::{encode, Token};
use ink_prelude::vec::Vec;
use scale::{Decode, Encode};
use tiny_keccak::{Hasher, Keccak};

#[derive(Encode, Decode)]
pub struct ExtData {
    pub recipient: Vec<u8>,
    pub relayer: Vec<u8>,
    pub ext_amount: i128,
    pub fee: u128,
    pub encrypted_output1: Vec<u8>,
    pub encrypted_output2: Vec<u8>,
}

impl ExtData {
    pub fn new(
        recipient: Vec<u8>,
        relayer: Vec<u8>,
        ext_amount: i128,
        fee: u128,
        encrypted_output1: Vec<u8>,
        encrypted_output2: Vec<u8>,
    ) -> ExtData {
        ExtData {
            fee,
            ext_amount,
            recipient,
            relayer,
            encrypted_output1,
            encrypted_output2,
        }
    }

    pub fn get_encode(&self) -> Vec<u8> {
        let codec = self.encode_abi();
        let mut keccak = Keccak::v256();
        keccak.update(codec.as_slice());
        let mut output = [0u8; 32];
        keccak.finalize(&mut output);
        let field_res = Bn254Fr::from_le_bytes_mod_order(&output);
        let value = field_res.into_repr().to_bytes_le();

        value
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait IntoAbiToken {
    fn into_abi(&self) -> Token;
    fn encode_abi(&self) -> Vec<u8> {
        let token = self.into_abi();
        encode(&[token])
    }
}

impl IntoAbiToken for i128 {
    fn into_abi(&self) -> Token {
        let bytes = self.encode();
        let mut bytes32: [u8; 32] = [0; 32];
        for (i, byte) in bytes.iter().enumerate() {
            bytes32[i] = *byte;
        }
        Token::Int(bytes32.into())
    }
}

impl IntoAbiToken for u128 {
    fn into_abi(&self) -> Token {
        let bytes = self.encode();
        let mut bytes32: [u8; 32] = [0; 32];
        for (i, byte) in bytes.iter().enumerate() {
            bytes32[i] = *byte;
        }
        Token::Uint(bytes32.into())
    }
}

impl IntoAbiToken for [u8; 32] {
    fn into_abi(&self) -> Token {
        Token::Bytes(self.to_vec())
    }
}

impl IntoAbiToken for ExtData {
    fn into_abi(&self) -> Token {
        let mut ext_data_args = Vec::new();
        let recipient = Token::Bytes(self.recipient.clone());
        let ext_amount = Token::Bytes(self.ext_amount.encode());
        let relayer = Token::Bytes(self.relayer.clone());
        let fee = Token::Bytes(self.fee.encode());
        let encrypted_output1 = Token::Bytes(self.encrypted_output1.clone());
        let encrypted_output2 = Token::Bytes(self.encrypted_output2.clone());
        ext_data_args.push(recipient);
        ext_data_args.push(relayer);
        ext_data_args.push(ext_amount);
        ext_data_args.push(fee);
        ext_data_args.push(encrypted_output1);
        ext_data_args.push(encrypted_output2);

        Token::Tuple(ext_data_args)
    }
}
