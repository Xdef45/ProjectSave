use argon2::{Argon2, Params};
use base64::prelude::*;

const MEMORY_COST: u32 = 64*1024;
const ITERATION_COST: u32 = 3;
const PARALLELISM_COST: u32 = 4;
const HASH_LENGTH: usize = 32;

fn byte_hash(password: &String, salt: &String, param: Params) -> [u8; HASH_LENGTH]{
    let password = password.as_bytes();
    let salt = salt.as_bytes();
    let mut out= [0u8; HASH_LENGTH];
    let _ = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, param).hash_password_into(password, salt, &mut out);
    return out;
}

/* $argon2id$v=19$m=65536,t=3,p=4$c2FqaXVoeWd2bHQ$xDxVByptQM0YaYZNsj1+LauIiAoWsfdTkRZVnsEKXzE */
pub fn base64_full_hash(password: &String, salt: &String) -> String{
    let output_len: Option<usize> = Some(HASH_LENGTH);
    let param: Params= match argon2::Params::new(MEMORY_COST, ITERATION_COST, PARALLELISM_COST, output_len) {
        Ok(v) => v,
        Err(_err) => return String::from("problème")
    };
    let base64_salt= BASE64_STANDARD.encode(salt.as_bytes());
    let byte_hash = byte_hash(password, salt, param);
    let base64_hash= BASE64_STANDARD.encode(byte_hash);
    let base64_full_hash = String::from(format!("$argon2id$v=19$m={},t={},p={}${}${}",MEMORY_COST, ITERATION_COST, PARALLELISM_COST, base64_salt,base64_hash ));
    return base64_full_hash;
}
