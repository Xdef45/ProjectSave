use argon2::{Argon2, Params};


const MEMORY_COST: u32 = 64*1024;
const ITERATION_COST: u32 = 3;
const PARALLELISM_COST: u32 = 4;
const HASH_LENGTH: usize = 32;

pub async fn create_kdf(password: &String, salt: &String) -> [u8; HASH_LENGTH]{
    let output_len: Option<usize> = Some(HASH_LENGTH);
    let param: Params= argon2::Params::new(MEMORY_COST, ITERATION_COST, PARALLELISM_COST, output_len).expect("problème");
    let password = password.as_bytes();
    let salt = salt.as_bytes();
    let mut out= [0u8; HASH_LENGTH];
    let _ = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, param).hash_password_into(password, salt, &mut out);
    return out;
}

