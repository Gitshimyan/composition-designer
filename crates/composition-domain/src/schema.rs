use std::fmt;

/// 声明格式版本。v0.1 只支持 `1`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    pub const V1: Self = Self(1);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub const fn is_supported(self) -> bool {
        self.0 == 1
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_is_supported_and_others_are_not() {
        assert!(SchemaVersion::V1.is_supported());
        assert!(SchemaVersion::new(1).is_supported());
        assert!(!SchemaVersion::new(2).is_supported());
        assert_eq!(SchemaVersion::default(), SchemaVersion::V1);
        assert_eq!(SchemaVersion::new(1).to_string(), "1");
    }
}
