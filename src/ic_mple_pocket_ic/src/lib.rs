use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use std::{env, fs};

use ::pocket_ic::{nonblocking::PocketIc, PocketIcBuilder, WasmResult};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use flate2::read::GzDecoder;
use log::*;

pub mod pocket_ic {
    pub use pocket_ic::*;
}

const POCKET_IC_SERVER_VERSION: &str = "7.0.0";

/// Returns the pocket-ic client.
/// If pocket-ic server binary is not present, it downloads it and sets
/// the `POCKET_IC_BIN` environment variable accordingly.
/// See: https://crates.io/crates/pocket-ic
///
/// The temp directory is used to store the binary.
///
/// To use custom server binary, the `POCKET_IC_BIN` environment variable should be set and
/// point to the binary. Also, the binary should be executable.
///
/// It supports only linux and macos.
pub fn get_pocket_ic_client() -> PocketIcBuilder {
    static INITIALIZATION_STATUS: OnceLock<bool> = OnceLock::new();

    let status: &bool = INITIALIZATION_STATUS.get_or_init(|| {
        if check_custom_pocket_ic_initialized() {
            // Custom server binary found. Let's use it.
            return true;
        };

        if let Some(binary_path) = dbg!(check_default_pocket_ic_binary_exist()) {
            // Default server binary found. Let's use it.
            env::set_var("POCKET_IC_BIN", binary_path);
            return true;
        }

        // Server binary not found. Let's download it.
        let mut target_dir = env::var("POCKET_IC_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_pocket_ic_server_binary_path());

        target_dir.pop();

        let binary_path = download_binary(target_dir);
        env::set_var("POCKET_IC_BIN", binary_path);

        true
    });

    if !*status {
        panic!("pocket-ic is not initialized");
    }

    PocketIcBuilder::new()
}

fn default_pocket_ic_server_dir() -> PathBuf {
    env::temp_dir()
        .join("pocket-ic-server")
        .join(POCKET_IC_SERVER_VERSION)
}

fn default_pocket_ic_server_binary_path() -> PathBuf {
    default_pocket_ic_server_dir().join("pocket-ic")
}

fn check_custom_pocket_ic_initialized() -> bool {
    if let Ok(path) = env::var("POCKET_IC_BIN") {
        return Path::new(&path).exists();
    }
    false
}

fn check_default_pocket_ic_binary_exist() -> Option<PathBuf> {
    let path = default_pocket_ic_server_binary_path();
    path.exists().then_some(path)
}

fn download_binary(pocket_ic_dir: PathBuf) -> PathBuf {
    let platform = match env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        _ => panic!("pocket-ic requires linux or macos"),
    };

    let download_url = format!("https://github.com/dfinity/pocketic/releases/download/{POCKET_IC_SERVER_VERSION}/pocket-ic-x86_64-{platform}.gz");

    // Download file
    let gz_binary = {
        info!("downloading pocket-ic server binary from: {download_url}");

        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap()
            .get(download_url)
            .send()
            .unwrap();

        response
            .bytes()
            .expect("pocket-ic server binary should be downloaded correctly")
    };

    let gz_data_cursor = Cursor::new(gz_binary);
    let binary_file_path = pocket_ic_dir.join("pocket-ic");
    fs::create_dir_all(&pocket_ic_dir)
        .expect("pocket-ic server path directories should be created");

    // unzip file
    {
        info!("unzip pocket-ic.gz to [{binary_file_path:?}]");

        let mut tar = GzDecoder::new(gz_data_cursor);
        let mut temp = vec![];
        tar.read_to_end(&mut temp)
            .expect("pocket-ic.gz should be decompressed");

        fs::write(&binary_file_path, temp)
            .expect("pocket-ic server binary should be written to file");

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::prelude::PermissionsExt;
            let mut perms = std::fs::metadata(&binary_file_path).unwrap().permissions();
            perms.set_mode(0o770);
            std::fs::set_permissions(&binary_file_path, perms).unwrap();
        }
    }

    binary_file_path
}

/// Query a canister method and return the result.
pub async fn query_call<T: CandidType, Result>(
    pocket_ic: &PocketIc,
    sender: Principal,
    canister_id: Principal,
    method: &str,
    payload: &T,
) -> Result
where
    for<'a> Result: CandidType + Deserialize<'a>,
{
    let payload = Encode!(payload).expect("failed to encode item to candid");
    let res = match pocket_ic
        .query_call(canister_id, sender, method, payload)
        .await
        .unwrap()
    {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(e) => panic!("Unexpected reject: {:?}", e),
    };

    Decode!(&res, Result).expect("failed to decode item from candid")
}

/// Call an update canister method and return the candid result.
pub async fn update_call<T: CandidType, Result>(
    pocket_ic: &PocketIc,
    sender: Principal,
    canister_id: Principal,
    method: &str,
    payload: &T,
) -> Result
where
    for<'a> Result: CandidType + Deserialize<'a>,
{
    let payload = Encode!(payload).expect("failed to encode item to candid");
    let res = match pocket_ic
        .update_call(canister_id, sender, method, payload)
        .await
        .unwrap()
    {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(e) => panic!("Unexpected reject: {:?}", e),
    };

    Decode!(&res, Result).expect("failed to decode item from candid")
}

/// Load wasm bytes from a file.
pub fn load_wasm_bytes(wasm_path: &str) -> Vec<u8> {
    let path = PathBuf::from(wasm_path);
    let mut f = File::open(path).expect("File does not exists");
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer)
        .expect("Could not read file content");

    buffer
}

#[test]
fn should_initialize_pocket_ic() {
    get_pocket_ic_client();
}
