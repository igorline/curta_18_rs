use ethers_core::{
    abi::{self, Token},
    types::{Address, U256, U64},
};
use tiny_keccak::{Hasher, Keccak};

use crate::CHALLENGE_ADDRESS;

pub fn get_clone_address(block_number: U64, sigil: U256) -> Address {
    let mut salt = [0u8; 32];
    let salt_val = abi::encode_packed(&[
        Token::Uint(U256::from(block_number.as_u64())),
        Token::Uint(sigil),
    ])
    .unwrap();
    let salt_u256 = U256::from_big_endian(&salt_val);
    salt_u256.to_big_endian(&mut salt);
    let init_code = "602c3d8160093d39f33d3d3d3d363d3d37363d73946c6169cb9fe5a46abe14e82a40093cbec502485af43d3d93803e602a57fd5bf3";
    let mut init_code_hex = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(&hex::decode(init_code).unwrap());
    hasher.finalize(&mut init_code_hex);

    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    let mut bytes = vec![0xff];
    bytes.extend_from_slice(&hex::decode(CHALLENGE_ADDRESS).unwrap());
    bytes.extend_from_slice(&salt);
    bytes.extend_from_slice(&init_code_hex);
    hasher.update(&bytes);
    hasher.finalize(&mut output);

    let mut output_vec = output.to_vec();
    output_vec.reverse();
    output_vec.truncate(20);
    output_vec.reverse();

    Address::from_slice(&output_vec)
}
