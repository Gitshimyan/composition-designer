use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// 实例配置值。v0.1 不支持 TOML table/object。
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Boolean(_) => "boolean",
            Self::Array(_) => "array",
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<RawConfigValue>),
}

impl<'de> Deserialize<'de> for ConfigValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(RawConfigValue::deserialize(deserializer)?.into_config())
    }
}

impl RawConfigValue {
    fn into_config(self) -> ConfigValue {
        match self {
            Self::String(value) => ConfigValue::String(value),
            Self::Integer(value) => ConfigValue::Integer(value),
            Self::Float(value) => ConfigValue::Float(value),
            Self::Boolean(value) => ConfigValue::Boolean(value),
            Self::Array(values) => {
                ConfigValue::Array(values.into_iter().map(Self::into_config).collect())
            }
        }
    }
}

impl Serialize for ConfigValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::String(value) => serializer.serialize_str(value),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::Float(value) => {
                if value.is_finite() {
                    serializer.serialize_f64(*value)
                } else if value.is_nan() {
                    serializer.serialize_str("nan")
                } else if value.is_sign_positive() {
                    serializer.serialize_str("inf")
                } else {
                    serializer.serialize_str("-inf")
                }
            }
            Self::Boolean(value) => serializer.serialize_bool(*value),
            Self::Array(values) => values.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_type_names_and_accessors() {
        assert_eq!(ConfigValue::String("x".into()).type_name(), "string");
        assert_eq!(ConfigValue::Integer(1).type_name(), "integer");
        assert_eq!(ConfigValue::Float(1.0).type_name(), "float");
        assert_eq!(ConfigValue::Boolean(true).type_name(), "boolean");
        assert_eq!(ConfigValue::Array(vec![]).type_name(), "array");
        assert_eq!(ConfigValue::Integer(7).as_integer(), Some(7));
        assert!(ConfigValue::Integer(7).as_float().is_none());
    }
}
