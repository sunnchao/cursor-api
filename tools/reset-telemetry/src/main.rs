use rand::RngCore;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn main() -> std::io::Result<()> {
    // 获取用户主目录路径
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap();

    // 构建storage.json的路径
    let db_path = if cfg!(target_os = "windows") {
        PathBuf::from(home_dir.clone())
            .join(r"AppData\Roaming\Cursor\User\globalStorage\storage.json")
    } else if cfg!(target_os = "linux") {
        PathBuf::from(home_dir.clone()).join(".config/Cursor/User/globalStorage/storage.json")
    } else {
        PathBuf::from(home_dir.clone())
            .join("Library/Application Support/Cursor/User/globalStorage/storage.json")
    };

    // 构建machineid文件的路径
    let machine_id_path = if cfg!(target_os = "windows") {
        PathBuf::from(home_dir).join(r"AppData\Roaming\Cursor\machineid")
    } else if cfg!(target_os = "linux") {
        PathBuf::from(home_dir).join(".config/Cursor/machineid")
    } else {
        PathBuf::from(home_dir).join("Library/Application Support/Cursor/machineid")
    };

    // 读取并更新storage.json
    let mut content: Value = if db_path.exists() {
        let content = fs::read_to_string(&db_path)?;
        serde_json::from_str(&content)?
    } else {
        json!({})
    };

    // 生成新的遥测ID
    content["telemetry.macMachineId"] = json!(generate_sha256_hash());
    content["telemetry.sqmId"] = json!(generate_sqm_id());
    content["telemetry.machineId"] = json!(generate_sha256_hash());
    content["telemetry.devDeviceId"] = json!(generate_device_id());

    // 写入更新后的storage.json
    fs::write(&db_path, serde_json::to_string_pretty(&content)?)?;

    // 更新machineid文件
    fs::write(&machine_id_path, generate_device_id())?;

    println!("遥测ID已重置成功！");
    Ok(())
}

fn generate_sha256_hash() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    let hash = Sha256::digest(&bytes);
    format!("{:x}", hash)
}

fn generate_sqm_id() -> String {
    use hex::ToHex as _;
    Uuid::new_v4().braced().encode_hex_upper()
}

fn generate_device_id() -> String {
    Uuid::new_v4().to_string()
}
