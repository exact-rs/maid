use maid::log::prelude::*;
use merkle_hash::{Algorithm, MerkleTree};
use std::path::Path;

const DEFAULT_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn bytes_to_hex(bytes: impl AsRef<[u8]>) -> &'static str {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    static mut HEX_BUFFER: [u8; 1024] = [0; 1024];

    let bytes = bytes.as_ref();
    let len = bytes.len();

    // Safety: we ensure exclusive access and stay within bounds
    unsafe {
        let buf = &mut HEX_BUFFER[..(len * 2)];

        for (i, &byte) in bytes.iter().enumerate() {
            let idx = i * 2;
            buf[idx] = TABLE[(byte >> 4) as usize];
            buf[idx + 1] = TABLE[(byte & 0xf) as usize];
        }

        std::str::from_utf8_unchecked(&buf[..(len * 2)])
    }
}

pub(crate) fn create_hash<'hash>(path: impl AsRef<Path>) -> &'hash str {
    let path = path.as_ref();

    if !path.exists() {
        warn!("Path does not exist: {}", path.display());
        return DEFAULT_HASH;
    }

    let path = match path.to_str() {
        Some(path) => path,
        None => return DEFAULT_HASH,
    };

    match MerkleTree::builder(path).algorithm(Algorithm::Blake3).hash_names(false).build() {
        Ok(tree) => {
            let hash = bytes_to_hex(tree.root.item.hash);
            debug!(path, "Successfully created tree hash");
            return hash;
        }
        Err(err) => {
            warn!(%err, path, "Failed to create tree hash");
            return DEFAULT_HASH;
        }
    };
}
