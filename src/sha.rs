use sha2::Digest;
use sha2::Sha256;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
struct Sha256Hash {
    digest: [u8; 32],
}

impl Sha256Hash {
    pub fn new(digest: [u8; 32]) -> Self {
        Self { digest }
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.digest
    }
}

pub(crate) fn hash(data: &[u8]) -> Sha256Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    Sha256Hash::new(digest.into())
}

pub(crate) fn hash_string(data: &str) -> Sha256Hash {
    hash(data.as_bytes())
}

pub(crate) fn hash_path(path: &Path) -> Sha256Hash {
    let data = std::fs::read(path).unwrap();
    hash(&data)
}

#[test]
fn test_hash() {
    use hex_literal::hex;
    let text = b"hello world";
    let expected = hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    assert_eq!(hash(text).as_bytes(), &expected);
}