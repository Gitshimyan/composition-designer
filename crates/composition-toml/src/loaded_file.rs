use std::ops::Range;

use composition_domain::{Position, SourceRange, SourceRef};
use toml_edit::ImDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRole {
    Project,
    Composition,
    Module,
}

/// 一个已读取的声明文件：保留原文、可选解析文档与字节 → 行列映射。
pub struct LoadedFile {
    relative_path: String,
    text: String,
    line_starts: Vec<usize>,
    document: Option<ImDocument<String>>,
    role: FileRole,
}

impl LoadedFile {
    pub fn new(
        relative_path: impl Into<String>,
        text: String,
        document: Option<ImDocument<String>>,
        role: FileRole,
    ) -> Self {
        let line_starts = compute_line_starts(&text);
        Self {
            relative_path: relative_path.into(),
            text,
            line_starts,
            document,
            role,
        }
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn document(&self) -> Option<&ImDocument<String>> {
        self.document.as_ref()
    }

    pub fn role(&self) -> FileRole {
        self.role
    }

    /// 字节 offset → 1-based 行列；越界或非字符边界返回 `None`。
    pub fn line_col(&self, offset: usize) -> Option<Position> {
        if offset > self.text.len() || !self.text.is_char_boundary(offset) {
            return None;
        }
        let line_index = self
            .line_starts
            .partition_point(|start| *start <= offset)
            .checked_sub(1)?;
        let line_start = self.line_starts[line_index];
        let column = self.text[line_start..offset].chars().count() + 1;
        Position::new(
            u32::try_from(line_index + 1).ok()?,
            u32::try_from(column).ok()?,
        )
    }

    /// 字节 span → 半开 `SourceRange`。
    pub fn range(&self, span: Option<Range<usize>>) -> Option<SourceRange> {
        let span = span?;
        if span.end < span.start {
            return None;
        }
        let start = self.line_col(span.start)?;
        let end = self.line_col(span.end)?;
        SourceRange::new(start, end)
    }

    pub fn source_ref(&self, span: Option<Range<usize>>) -> SourceRef {
        match self.range(span) {
            Some(range) => SourceRef::new(&self.relative_path).with_range(range),
            None => SourceRef::new(&self.relative_path),
        }
    }
}

fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(text: &str) -> LoadedFile {
        LoadedFile::new("x.toml", text.to_owned(), None, FileRole::Module)
    }

    #[test]
    fn maps_byte_offsets_with_unicode_columns() {
        let file = file("a\nβx\n");
        assert_eq!(file.line_col(0), Position::new(1, 1));
        assert_eq!(file.line_col(2), Position::new(2, 1));
        assert_eq!(file.line_col(4), Position::new(2, 2));
        assert_eq!(file.line_col(5), Position::new(2, 3));
        assert_eq!(file.line_col(6), Position::new(3, 1));
    }

    #[test]
    fn handles_crlf_as_one_newline() {
        let file = file("a\r\nb\r\n");
        assert_eq!(file.line_col(3), Position::new(2, 1));
        assert_eq!(file.line_col(4), Position::new(2, 2));
    }

    #[test]
    fn rejects_out_of_range_and_non_boundary_offsets() {
        let file = file("aβ");
        assert!(file.line_col(4).is_none());
        assert!(file.line_col(1).is_some());
        assert!(file.line_col(2).is_none());
    }

    #[test]
    fn builds_half_open_ranges_and_source_refs() {
        let file = file("a\nb\n");
        let range = file.range(Some(0..3)).unwrap();
        assert_eq!(range.start, Position::new(1, 1).unwrap());
        assert_eq!(range.end, Position::new(2, 2).unwrap());
        assert_eq!(file.source_ref(Some(0..3)).to_string(), "x.toml:1:1");
    }
}
