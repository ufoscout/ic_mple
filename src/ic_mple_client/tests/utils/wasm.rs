use std::fs::File;
use std::io::Read;
use std::sync::OnceLock;

pub fn get_test_canister_bytecode() -> Vec<u8> {
    static CANISTER_BYTECODE: OnceLock<Vec<u8>> = OnceLock::new();
    CANISTER_BYTECODE
        .get_or_init(|| load_canister_bytecode("test_canister.wasm"))
        .to_owned()
}

fn load_canister_bytecode(wasm_name: &str) -> Vec<u8> {

    let path = format!("../../target/wasm32-unknown-unknown/release/{wasm_name}");

    let mut f = File::open(path).expect("File does not exists");

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect("Could not read file content");

    buffer
}
