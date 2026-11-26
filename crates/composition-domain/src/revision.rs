use std::fmt;

/// `sha256:` + 64 位小写十六进制。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceDigest(String);

impl SourceDigest {
    pub fn parse(value: &str) -> Option<Self> {
        let hex = value.strip_prefix("sha256:")?;
        if is_lower_hex_64(hex) {
            Some(Self(value.to_owned()))
        } else {
            None
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        if is_lower_hex_64(hex) {
            Some(Self(format!("sha256:{hex}")))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Revision {
    pub seq: u64,
    pub source_digest: Option<SourceDigest>,
}

impl Revision {
    pub fn new(seq: u64, source_digest: Option<SourceDigest>) -> Self {
        Self { seq, source_digest }
    }

    pub fn initial(source_digest: Option<SourceDigest>) -> Self {
        Self::new(0, source_digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn accepts_prefixed_lowercase_hex() {
        let digest = SourceDigest::parse(&format!("sha256:{HEX}")).unwrap();
        assert_eq!(digest.as_str(), format!("sha256:{HEX}"));
        assert_eq!(SourceDigest::from_hex(HEX), Some(digest));
    }

    #[test]
    fn rejects_bad_digests() {
        assert!(SourceDigest::parse(HEX).is_none());
        assert!(SourceDigest::parse(&format!("sha256:{}", HEX.to_uppercase())).is_none());
        assert!(SourceDigest::parse("sha256:abc").is_none());
        assert!(SourceDigest::from_hex("zz").is_none());
    }

    #[test]
    fn initial_revision_starts_at_zero() {
        let digest = SourceDigest::from_hex(HEX);
        let revision = Revision::initial(digest.clone());
        assert_eq!(revision.seq, 0);
        assert_eq!(revision.source_digest, digest);
    }
}
