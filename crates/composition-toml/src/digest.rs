use composition_domain::SourceDigest;
use sha2::{Digest, Sha256};

/// schema §9：按路径排序，逐个写入 `path + 0x00 + len + 0x00 + bytes + 0x00`。
pub fn compute_source_digest<'a, I>(files: I) -> Option<SourceDigest>
where
    I: IntoIterator<Item = (&'a str, &'a [u8])>,
{
    let mut entries: Vec<(&str, &[u8])> = files.into_iter().collect();
    if entries.is_empty() {
        return None;
    }
    entries.sort_by(|left, right| left.0.cmp(right.0));

    let mut hasher = Sha256::new();
    for (path, bytes) in entries {
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(bytes.len().to_string().as_bytes());
        hasher.update([0]);
        hasher.update(bytes);
        hasher.update([0]);
    }
    let hex = format!("{:x}", hasher.finalize());
    SourceDigest::from_hex(&hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_independent_of_input_order() {
        let forward = compute_source_digest([("a.toml", b"a".as_slice()), ("b.toml", b"b")]);
        let reverse = compute_source_digest([("b.toml", b"b".as_slice()), ("a.toml", b"a")]);
        assert_eq!(forward, reverse);
    }

    #[test]
    fn matches_known_vector() {
        let digest =
            compute_source_digest([("a.toml", b"a".as_slice()), ("b.toml", b"b")]).expect("digest");
        assert_eq!(
            digest.as_str(),
            "sha256:35d1d6cae692a1f7747c2cf597edda6962cf295319e641a9fa8936affba6bdb6"
        );
    }

    #[test]
    fn empty_input_has_no_digest() {
        assert!(compute_source_digest(std::iter::empty::<(&str, &[u8])>()).is_none());
    }
}
