use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer};

use crate::error::{DomainError, DomainResult};
use crate::error_code::ErrorCode;

/// 点分小写模块标识，例如 `shell.desktop`。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ModuleId(String);

impl ModuleId {
    pub fn parse(value: &str) -> DomainResult<Self> {
        if is_dotted_lower_kebab(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(invalid_identifier("module id", value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ModuleId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for ModuleId {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Port 类型标识，词法与模块 ID 相同，例如 `string` 或 `contract.task.state.v1`。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct TypeId(String);

impl TypeId {
    pub fn parse(value: &str) -> DomainResult<Self> {
        if is_dotted_lower_kebab(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(invalid_identifier("type id", value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TypeId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for TypeId {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 局部标识：实例、Binding、Connection、属性、Slot、Port、依赖。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct LocalId(String);

impl LocalId {
    pub fn parse(value: &str) -> DomainResult<Self> {
        if is_local_id(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(invalid_identifier("local id", value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for LocalId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for LocalId {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 精确 `MAJOR.MINOR.PATCH` 版本，无前导零、无预发布后缀。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(value: &str) -> DomainResult<Self> {
        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() != 3 {
            return Err(invalid_version(value));
        }
        let mut numbers = [0_u32; 3];
        for (slot, part) in numbers.iter_mut().zip(parts.iter()) {
            let valid = !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0'));
            if !valid {
                return Err(invalid_version(value));
            }
            *slot = part.parse::<u32>().map_err(|_| invalid_version(value))?;
        }
        Ok(Self::new(numbers[0], numbers[1], numbers[2]))
    }
}

impl FromStr for Version {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn is_dotted_lower_kebab(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) if first.is_ascii_lowercase() => {}
                _ => return false,
            }
            chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        })
}

fn is_local_id(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
}

fn invalid_identifier(kind: &str, value: &str) -> DomainError {
    DomainError::new(
        ErrorCode::SchemaInvalidIdentifier,
        format!("invalid {kind} {value:?}"),
    )
}

fn invalid_version(value: &str) -> DomainError {
    DomainError::new(
        ErrorCode::SchemaInvalidVersion,
        format!("invalid exact version {value:?}; expected MAJOR.MINOR.PATCH"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_ids_accept_dotted_lower_kebab() {
        for value in [
            "a",
            "shell.desktop",
            "leaf.widget",
            "a.b-c",
            "contract.task.state.v1",
        ] {
            let id = ModuleId::parse(value).expect(value);
            assert_eq!(id.as_str(), value);
            assert_eq!(id.to_string(), value);
        }
    }

    #[test]
    fn module_ids_reject_bad_shapes() {
        for value in ["", ".a", "a.", "A", "a..b", "a_b", "1a", "a b", "a/b", "a."] {
            let error = ModuleId::parse(value).expect_err(value);
            assert_eq!(error.code(), ErrorCode::SchemaInvalidIdentifier);
        }
    }

    #[test]
    fn local_ids_accept_underscore_and_hyphen() {
        for value in ["app", "state-1", "a_b", "card-a"] {
            let id = LocalId::parse(value).expect(value);
            assert_eq!(id.as_str(), value);
        }
    }

    #[test]
    fn local_ids_reject_bad_shapes() {
        for value in ["", "1a", "A", "a.b", "a b", "-a", "_a"] {
            let error = LocalId::parse(value).expect_err(value);
            assert_eq!(error.code(), ErrorCode::SchemaInvalidIdentifier);
        }
    }

    #[test]
    fn type_ids_use_module_grammar() {
        assert!(TypeId::parse("string").is_ok());
        assert!(TypeId::parse("contract.task.state.v1").is_ok());
        assert!(TypeId::parse("String").is_err());
    }

    #[test]
    fn versions_accept_exact_semver() {
        for value in ["0.0.0", "1.0.0", "10.20.30"] {
            let version = Version::parse(value).expect(value);
            assert_eq!(version.to_string(), value);
        }
    }

    #[test]
    fn versions_reject_ranges_prerelease_and_leading_zeroes() {
        for value in [
            "1.0",
            "1.0.0.0",
            "01.0.0",
            "1.00.0",
            "1.0.0-alpha",
            "-1.0.0",
            "1.0.0 ",
            "latest",
        ] {
            let error = Version::parse(value).expect_err(value);
            assert_eq!(error.code(), ErrorCode::SchemaInvalidVersion);
        }
    }

    #[test]
    fn version_overflow_is_rejected() {
        assert!(Version::parse("4294967296.0.0").is_err());
    }

    #[test]
    fn module_ids_sort_lexicographically() {
        let mut ids = [
            ModuleId::parse("shell.desktop").unwrap(),
            ModuleId::parse("leaf.widget").unwrap(),
            ModuleId::parse("ui.page").unwrap(),
        ];
        ids.sort();
        let sorted: Vec<&str> = ids.iter().map(ModuleId::as_str).collect();
        assert_eq!(sorted, ["leaf.widget", "shell.desktop", "ui.page"]);
    }
}
