use std::collections::HashMap;

use avro_rs::{from_avro_datum, Schema};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use bytes::Bytes;
use sha2::{Digest, Sha256};

use crate::error::IndexerError;

use super::models::{DataItem, Tag};

const HEADER_START: usize = 32;

#[derive(Debug)]
struct SigType {
    sig_length: usize,
    pub_length: usize,
}

lazy_static::lazy_static! {
    static ref SIG_TYPES: HashMap<&'static str, SigType> = {
        let mut m = HashMap::new();
        m.insert("ARWEAVE", SigType { sig_length: 512, pub_length: 512 });
        m.insert("ED25519", SigType { sig_length: 64, pub_length: 32 });
        m.insert("ETHEREUM", SigType { sig_length: 65, pub_length: 65 });
        m.insert("SOLANA", SigType { sig_length: 64, pub_length: 32 });
        m.insert("INJECTEDAPTOS", SigType { sig_length: 64, pub_length: 32 });
        m.insert("MULTIAPTOS", SigType { sig_length: 64 * 32 + 4, pub_length: 32 * 32 + 1 });
        m.insert("TYPEDETHEREUM", SigType { sig_length: 65, pub_length: 42 });
        m
    };
}

fn byte_array_to_long(byte_array: &[u8]) -> u128 {
    let mut value = 0u128;
    for &byte in byte_array.iter().rev() {
        value = value.checked_mul(256).unwrap() + byte as u128;
    }
    value
}

pub fn get_item_count(bytes: &[u8]) -> u128 {
    byte_array_to_long(&bytes[..32])
}

fn get_bundle_start(item_count: u128) -> usize {
    HEADER_START + 64 * item_count as usize
}

fn get_signature_type(binary: &[u8]) -> Option<&'static str> {
    const SIGNATURE_CONFIG: [&str; 7] = [
        "ARWEAVE",
        "ED25519",
        "ETHEREUM",
        "SOLANA",
        "INJECTEDAPTOS",
        "MULTIAPTOS",
        "TYPEDETHEREUM",
    ];
    let signature_type_val = byte_array_to_long(&binary[0..2]);
    if signature_type_val > 0 && signature_type_val <= SIGNATURE_CONFIG.len() as u128 {
        Some(SIGNATURE_CONFIG[(signature_type_val - 1) as usize])
    } else {
        None
    }
}

fn get_signature_length(signature_type: &str) -> usize {
    SIG_TYPES.get(signature_type).unwrap().sig_length
}

fn get_raw_signature(binary: &[u8], signature_length: usize) -> Vec<u8> {
    binary[2..2 + signature_length].to_vec()
}

fn get_owner_length(signature_type: &str) -> usize {
    SIG_TYPES.get(signature_type).unwrap().pub_length
}

fn get_raw_owner(binary: &[u8], signature_length: usize, owner_length: usize) -> &[u8] {
    &binary[2 + signature_length..2 + signature_length + owner_length]
}

fn get_ids(binary: &[u8], item_count: u128) -> Vec<String> {
    let mut ids = Vec::new();
    for i in (HEADER_START..HEADER_START + 64 * item_count as usize).step_by(64) {
        let bundle_id = &binary[i + 32..i + 64];
        if bundle_id.is_empty() {
            panic!("Invalid bundle, id specified in headers doesn't exist");
        }
        ids.push(URL_SAFE_NO_PAD.encode(bundle_id));
    }
    ids
}

fn owner_to_address(raw_owner: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_owner);
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

fn get_target_start(signature_length: usize, owner_length: usize) -> usize {
    2 + signature_length + owner_length
}

fn get_tags_start(binary: &[u8], signature_length: usize, owner_length: usize) -> usize {
    let target_start = get_target_start(signature_length, owner_length);
    let target_present = binary[target_start] == 1;
    let mut tags_start = target_start + if target_present { 33 } else { 1 };
    let anchor_present = binary[tags_start] == 1;
    tags_start += if anchor_present { 33 } else { 1 };
    tags_start
}

fn get_tags(binary: &[u8], signature_length: usize, owner_length: usize) -> Vec<Tag> {
    let tags_start = get_tags_start(binary, signature_length, owner_length);
    let tags_count = byte_array_to_long(&binary[tags_start..tags_start + 8]) as usize;
    if tags_count == 0 {
        return Vec::new();
    }
    let tags_size = byte_array_to_long(&binary[tags_start + 8..tags_start + 16]) as usize;
    let mut tags = &binary[tags_start + 16..tags_start + 16 + tags_size];

    let reader_raw_schema = r#"
    {
        "type": "array",
        "items": {
            "type": "record",
            "name": "Tag",
            "fields": [
                { "name": "name", "type": "bytes" },
                { "name": "value", "type": "bytes" }
            ]
        }
    }
    "#;

    let schema = Schema::parse_str(reader_raw_schema).unwrap();
    let v = from_avro_datum(&schema, &mut tags, Some(&schema)).unwrap();
    avro_rs::from_value(&v).unwrap()
}

fn get_data_item(binary: &[u8]) -> DataItem {
    let signature_type = get_signature_type(binary).unwrap().to_string();
    let signature_length = get_signature_length(&signature_type);
    let raw_signature = get_raw_signature(binary, signature_length);
    println!("Signature: {:?}", hex::encode(raw_signature));
    let owner_length = get_owner_length(&signature_type);
    let raw_owner = get_raw_owner(binary, signature_length, owner_length);
    DataItem {
        signature_type,
        owner: owner_to_address(raw_owner),
        tags: get_tags(binary, signature_length, owner_length),
        bundled_in: None,
        block_height: None,
        timestamp: None,
        tx_pos: 0,
        _id: "".to_string(),
    }
}

fn is_id_valid(id: &[u8]) -> bool {
    id[0..32].iter().all(|&x| x == 0)
}

fn get_items(
    binary_data: &[u8],
    bundled_in: Option<String>,
    block_height: Option<i64>,
    timestamp: Option<i64>,
) -> Vec<DataItem> {
    let item_count = get_item_count(binary_data);
    println!("Items: {}", item_count);
    let mut items = Vec::with_capacity(item_count as usize);
    let mut offset = 0;
    let bundle_start = get_bundle_start(item_count);
    let ids = get_ids(binary_data, item_count);
    for (counter, i) in (HEADER_START..HEADER_START + 64 * item_count as usize)
        .step_by(64)
        .enumerate()
    {
        let _offset = byte_array_to_long(&binary_data[i..i + 32]) as usize;
        if is_id_valid(&binary_data[i + 32..i + 64]) {
            eprintln!("Invalid ID at position: {}", counter);
            offset += _offset;
            continue;
        }

        let data_item_start = bundle_start + offset;
        let _bytes = &binary_data[data_item_start..data_item_start + _offset];
        offset += _offset;
        let mut item = get_data_item(_bytes);
        item._id = ids[counter].clone();
        item.bundled_in = bundled_in.clone();
        item.block_height = block_height;
        item.timestamp = timestamp;
        item.tx_pos = counter;
        items.push(item);
    }
    items
}

pub fn parse_bundle(bundle: Bytes) -> Result<Vec<DataItem>, IndexerError> {
    Ok(get_items(&bundle, None, None, None))
}
