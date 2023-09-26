use std::str::FromStr;

use tiny_keccak::{Hasher, Keccak};

fn get_bytes_slice_trimmed_zeros(bytes: &[u8]) -> &[u8] {
    let first_non_zero_index = bytes.iter().position(|&x| x != 0).unwrap_or(15);
    &bytes[first_non_zero_index..]
}

#[derive(Clone, Debug)]
pub enum Materia {
    Gold,
    Lead,
}

impl FromStr for Materia {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let materia = match s {
            "gold" => Materia::Gold,
            "lead" => Materia::Lead,
            _ => {
                return Err(eyre::eyre!("wrong materia specified"));
            }
        };
        Ok(materia)
    }
}

pub fn bruteforce_code_hash(materia: Materia, initial_runtime: &str) {
    let initial_bytes = hex::decode(initial_runtime).unwrap();

    let code_hash = match materia {
        Materia::Gold => "00901d",
        Materia::Lead => "001ead",
    };

    for val in 0..u128::MAX {
        let mut output = [0u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(&initial_bytes);
        let val_bytes = val.to_be_bytes();
        let trimmed = get_bytes_slice_trimmed_zeros(&val_bytes);
        hasher.update(trimmed);
        hasher.finalize(&mut output);

        println!("trying val: {}", val);
        let code_hash_string = hex::encode(output);
        if code_hash_string.starts_with(code_hash) {
            println!(
                "matching bytes: {}{}",
                hex::encode(&initial_bytes),
                hex::encode(trimmed)
            );
            println!("output: {:?}", hex::encode(output));
            println!(
                "deploycode: 60{:x}8060093d393df3{}{}",
                initial_bytes.len() + trimmed.len(),
                hex::encode(&initial_bytes),
                hex::encode(trimmed)
            );
            break;
        }
    }
}
