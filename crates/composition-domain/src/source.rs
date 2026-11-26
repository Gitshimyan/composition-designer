use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Position {
    /// 行列从 1 开始；0 不合法。
    pub fn new(line: u32, column: u32) -> Option<Self> {
        if line == 0 || column == 0 {
            None
        } else {
            Some(Self { line, column })
        }
    }
}

/// 半开区间：`start` 包含、`end` 不包含；`start <= end`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct SourceRange {
    pub start: Position,
    pub end: Position,
}

impl SourceRange {
    pub fn new(start: Position, end: Position) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }
}

/// 模型对象的来源：相对项目根的路径 + 可选范围。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct SourceRef {
    pub file: String,
    pub range: Option<SourceRange>,
}

impl SourceRef {
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            range: None,
        }
    }

    pub fn with_range(mut self, range: SourceRange) -> Self {
        self.range = Some(range);
        self
    }
}

impl fmt::Display for SourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.range {
            Some(range) => write!(
                f,
                "{}:{}:{}",
                self.file, range.start.line, range.start.column
            ),
            None => f.write_str(&self.file),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_are_one_based() {
        assert!(Position::new(1, 1).is_some());
        assert!(Position::new(0, 1).is_none());
        assert!(Position::new(1, 0).is_none());
    }

    #[test]
    fn ranges_require_ordered_endpoints() {
        let start = Position::new(1, 1).unwrap();
        let end = Position::new(2, 1).unwrap();
        assert!(SourceRange::new(start, end).is_some());
        assert!(SourceRange::new(start, start).is_some());
        assert!(SourceRange::new(end, start).is_none());
    }

    #[test]
    fn source_refs_display_file_and_position() {
        let file = "composition/main.toml";
        assert_eq!(SourceRef::new(file).to_string(), file);
        let range =
            SourceRange::new(Position::new(6, 1).unwrap(), Position::new(8, 14).unwrap()).unwrap();
        assert_eq!(
            SourceRef::new(file).with_range(range).to_string(),
            "composition/main.toml:6:1"
        );
    }
}
