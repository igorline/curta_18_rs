use rayon::prelude::*;
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

pub fn bruteforce_code_hash(start_code_hash: &[u8], initial_runtime: &str) {
    let initial_bytes = hex::decode(initial_runtime).unwrap();
    let start_code_hash_len = start_code_hash.len();

    let power_of_two_ranges = [
        (0u128..1 << 8),
        (1 << 8..1 << 16),
        (1 << 16..1 << 24),
        (1 << 24..1 << 32),
        (1 << 32..1 << 40),
        (1 << 40..1 << 48),
        (1 << 48..1 << 56),
        (1 << 56..1 << 64),
    ];

    for range in power_of_two_ranges {
        let val = range.into_par_iter().find_any(|val| {
            let mut output = [0u8; 32];
            let mut hasher = Keccak::v256();
            hasher.update(&initial_bytes);
            let val_bytes = val.to_be_bytes();
            let trimmed = get_bytes_slice_trimmed_zeros(&val_bytes);
            hasher.update(trimmed);
            hasher.finalize(&mut output);

            if &output[..start_code_hash_len] == start_code_hash {
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
                return true;
            }
            false
        });
        if val.is_some() {
            break;
        }
    }
}
