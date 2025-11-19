use sha2::{Sha256, Digest};

/// Hash a string with SHA256
pub fn hash_data(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
