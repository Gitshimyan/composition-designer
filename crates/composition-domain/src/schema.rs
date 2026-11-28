use std::fmt;

use serde::Serialize;

/// 声明格式版本。v2.1 起同时支持 `1`、`2` 与 `3`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    pub const V1: Self = Self(1);
    pub const V2: Self = Self(2);
    pub const V3: Self = Self(3);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub const fn is_supported(self) -> bool {
        self.0 == 1 || self.0 == 2 || self.0 == 3
    }

    pub const fn is_v2(self) -> bool {
        self.0 == 2
    }

    pub const fn is_v3(self) -> bool {
        self.0 == 3
    }

    /// v2.0 引入的接口/instance scope 语义在 v2 与 v3 中相同。
    pub const fn supports_interfaces(self) -> bool {
        self.0 >= 2
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
    fn v1_v2_v3_are_supported_and_others_are_not() {
        assert!(SchemaVersion::V1.is_supported());
        assert!(SchemaVersion::V2.is_supported());
        assert!(SchemaVersion::V3.is_supported());
        assert!(!SchemaVersion::new(0).is_supported());
        assert!(!SchemaVersion::new(4).is_supported());
        assert!(!SchemaVersion::V1.is_v2());
        assert!(SchemaVersion::V2.is_v2());
        assert!(!SchemaVersion::V2.is_v3());
        assert!(SchemaVersion::V3.is_v3());
        assert!(!SchemaVersion::V1.supports_interfaces());
        assert!(SchemaVersion::V2.supports_interfaces());
        assert!(SchemaVersion::V3.supports_interfaces());
        assert_eq!(SchemaVersion::default(), SchemaVersion::V1);
        assert_eq!(SchemaVersion::new(3).to_string(), "3");
    }
}
