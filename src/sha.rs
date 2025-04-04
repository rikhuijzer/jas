use crate::abort;
use crate::ShaArgs;
use sha2::Digest;
use sha2::Sha256;
use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Sha256Hash {
    pub digest: [u8; 32],
}

impl Display for Sha256Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.digest))
    }
}

impl Sha256Hash {
    pub fn new(digest: [u8; 32]) -> Self {
        Self { digest }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.digest
    }
    pub fn from_data(data: &[u8]) -> Sha256Hash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        Sha256Hash::new(digest.into())
    }
    #[allow(dead_code)]
    pub fn from_text(text: &str) -> Sha256Hash {
        Self::from_data(text.as_bytes())
    }
    pub fn from_path(path: &PathBuf) -> Sha256Hash {
        let data = std::fs::read(path).unwrap();
        Self::from_data(&data)
    }
}
impl PartialEq<str> for Sha256Hash {
    fn eq(&self, other: &str) -> bool {
        let other_bytes = hex::decode(other).unwrap();
        let other = other_bytes.as_slice();
        self.as_bytes() == other
    }
}

impl PartialEq<Sha256Hash> for String {
    fn eq(&self, other: &Sha256Hash) -> bool {
        *self == hex::encode(other.as_bytes())
    }
}

#[test]
fn test_hash() {
    let text = b"hello world";
    let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
    assert_eq!(Sha256Hash::from_data(text), *expected);
}

fn hash_from_path(path: &str) -> Sha256Hash {
    let path = PathBuf::from(path);
    if !path.exists() {
        abort(&format!("Path does not exist: {}", path.display()));
    }
    Sha256Hash::from_path(&path)
}

fn prefix_proto_if_needed(url: &str) -> String {
    if !url.starts_with("http") {
        format!("https://{}", url)
    } else {
        url.to_string()
    }
}

fn hash_from_url(url: &str) -> Sha256Hash {
    let url = prefix_proto_if_needed(url);
    tracing::info!("Downloading {}", url);
    let mut response = ureq::get(url).call().unwrap();
    let body = response.body_mut().read_to_vec().unwrap();
    Sha256Hash::from_data(&body)
}

pub fn run(args: &ShaArgs) {
    if let Some(path) = &args.path {
        let digest = hash_from_path(path);
        println!("{}", digest);
    } else if let Some(url) = &args.url {
        let digest = hash_from_url(url);
        println!("{}", digest);
    } else {
        abort("Specify either a path or a URL");
    }
}
