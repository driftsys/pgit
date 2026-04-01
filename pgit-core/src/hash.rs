use sha2::{Digest, Sha256};

/// Compute a SHA-256 digest of arbitrary bytes, returning a lowercase hex string.
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute a SHA-256 digest over a sequence of (path, content) pairs, sorted
/// by path for determinism.  Used to hash an entire package directory tree.
pub fn sha256_tree(entries: &mut Vec<(String, Vec<u8>)>) -> String {
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (path, content) in entries.iter() {
        // Feed path length + path + content length + content for framing.
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(content);
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_bytes_known_value() {
        // echo -n "hello" | sha256sum
        assert_eq!(
            sha256_bytes(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn tree_hash_is_order_independent() {
        let mut a = vec![
            ("b.txt".to_string(), b"world".to_vec()),
            ("a.txt".to_string(), b"hello".to_vec()),
        ];
        let mut b = vec![
            ("a.txt".to_string(), b"hello".to_vec()),
            ("b.txt".to_string(), b"world".to_vec()),
        ];
        assert_eq!(sha256_tree(&mut a), sha256_tree(&mut b));
    }

    #[test]
    fn tree_hash_changes_with_content() {
        let mut a = vec![("a.txt".to_string(), b"hello".to_vec())];
        let mut b = vec![("a.txt".to_string(), b"world".to_vec())];
        assert_ne!(sha256_tree(&mut a), sha256_tree(&mut b));
    }
}
