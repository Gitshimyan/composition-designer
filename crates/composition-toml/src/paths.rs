use composition_domain::ErrorCode;

/// 归一化相对路径，返回 POSIX 形式。允许 `.` / `..`，但越出根目录即错误。
pub fn normalize_relative_path(value: &str) -> Result<String, ErrorCode> {
    if value.is_empty() {
        return Err(ErrorCode::FilePathInvalid);
    }
    if value.starts_with('/') || has_drive_prefix(value) {
        return Err(ErrorCode::FilePathAbsolute);
    }
    if value.contains('\\') || value.ends_with('/') || value.contains("//") || value.contains('\0')
    {
        return Err(ErrorCode::FilePathInvalid);
    }

    let mut parts: Vec<&str> = Vec::new();
    for part in value.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                if parts.pop().is_none() {
                    return Err(ErrorCode::FilePathEscape);
                }
            }
            other => parts.push(other),
        }
    }
    if parts.is_empty() {
        return Err(ErrorCode::FilePathInvalid);
    }
    Ok(parts.join("/"))
}

pub(crate) fn is_nested_path(outer: &str, inner: &str) -> bool {
    inner
        .strip_prefix(outer)
        .is_some_and(|rest| rest.starts_with('/'))
}

fn has_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dot_and_parent_segments() {
        assert_eq!(
            normalize_relative_path("modules/./shell/../leaf").unwrap(),
            "modules/leaf"
        );
        assert_eq!(
            normalize_relative_path("composition/main.toml").unwrap(),
            "composition/main.toml"
        );
    }

    #[test]
    fn rejects_absolute_and_windows_paths() {
        for value in ["/abs", "C:/abs", "C:\\abs", "\\\\server\\share"] {
            let code = normalize_relative_path(value).unwrap_err();
            assert!(
                matches!(
                    code,
                    ErrorCode::FilePathAbsolute | ErrorCode::FilePathInvalid
                ),
                "{value}: {code}"
            );
        }
    }

    #[test]
    fn rejects_empty_segments_and_trailing_slash() {
        for value in ["", "a//b", "modules/", "a\\b", "."] {
            assert_eq!(
                normalize_relative_path(value).unwrap_err(),
                ErrorCode::FilePathInvalid,
                "{value}"
            );
        }
    }

    #[test]
    fn rejects_escape() {
        assert_eq!(
            normalize_relative_path("../outside").unwrap_err(),
            ErrorCode::FilePathEscape
        );
        assert_eq!(
            normalize_relative_path("a/../../outside").unwrap_err(),
            ErrorCode::FilePathEscape
        );
    }

    #[test]
    fn detects_nested_roots() {
        assert!(is_nested_path("modules", "modules/ui"));
        assert!(is_nested_path("modules", "modules/ui/shell"));
        assert!(!is_nested_path("modules", "modules-ui"));
        assert!(!is_nested_path("modules", "modules"));
    }
}
