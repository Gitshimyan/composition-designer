use std::cmp::Ordering;
use std::fmt;

use crate::error_code::{ErrorCode, Severity};
use crate::ids::{LocalId, ModuleId, Version};
use crate::source::SourceRange;

/// 诊断关联对象的稳定标识，格式见 domain-model-v1 §7.2。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn project() -> Self {
        Self("project".to_owned())
    }

    pub fn composition() -> Self {
        Self("composition".to_owned())
    }

    pub fn module(id: &ModuleId, version: &Version) -> Self {
        Self(format!("module:{id}@{version}"))
    }

    pub fn instance(id: &LocalId) -> Self {
        Self(format!("instance:{id}"))
    }

    pub fn binding(id: &LocalId) -> Self {
        Self(format!("binding:{id}"))
    }

    pub fn connection(id: &LocalId) -> Self {
        Self(format!("connection:{id}"))
    }

    pub fn property(module: &ModuleId, version: &Version, id: &LocalId) -> Self {
        Self(format!("property:{module}@{version}#{id}"))
    }

    pub fn slot(module: &ModuleId, version: &Version, id: &LocalId) -> Self {
        Self(format!("slot:{module}@{version}#{id}"))
    }

    pub fn port(module: &ModuleId, version: &Version, id: &LocalId) -> Self {
        Self(format!("port:{module}@{version}#{id}"))
    }

    pub fn dependency(module: &ModuleId, version: &Version, id: &LocalId) -> Self {
        Self(format!("dependency:{module}@{version}#{id}"))
    }

    pub fn file(path: &str) -> Self {
        Self(format!("file:{path}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: ErrorCode,
    pub severity: Severity,
    pub message: String,
    pub object_id: Option<ObjectId>,
    pub file: Option<String>,
    pub range: Option<SourceRange>,
    pub related: Vec<ObjectId>,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: code.severity(),
            message: message.into(),
            object_id: None,
            file: None,
            range: None,
            related: Vec::new(),
            hint: None,
        }
    }

    pub fn with_object_id(mut self, object_id: ObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn with_source(mut self, file: impl Into<String>, range: Option<SourceRange>) -> Self {
        self.file = Some(file.into());
        self.range = range;
        self
    }

    pub fn with_related(mut self, related: Vec<ObjectId>) -> Self {
        self.related = related;
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// schema §7.3 的稳定顺序：file → 起始行列 → code → object_id。
    pub fn compare_order(&self, other: &Self) -> Ordering {
        self.file
            .cmp(&other.file)
            .then_with(|| {
                self.range
                    .map(|range| (range.start.line, range.start.column))
                    .cmp(
                        &other
                            .range
                            .map(|range| (range.start.line, range.start.column)),
                    )
            })
            .then_with(|| self.code.cmp(&other.code))
            .then_with(|| {
                self.object_id
                    .as_ref()
                    .map(ObjectId::as_str)
                    .cmp(&other.object_id.as_ref().map(ObjectId::as_str))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Position;

    #[test]
    fn object_ids_follow_the_frozen_format() {
        let module = ModuleId::parse("shell.desktop").unwrap();
        let version = Version::parse("1.0.0").unwrap();
        let local = LocalId::parse("app").unwrap();
        assert_eq!(ObjectId::project().as_str(), "project");
        assert_eq!(ObjectId::composition().as_str(), "composition");
        assert_eq!(
            ObjectId::module(&module, &version).as_str(),
            "module:shell.desktop@1.0.0"
        );
        assert_eq!(ObjectId::instance(&local).as_str(), "instance:app");
        assert_eq!(ObjectId::binding(&local).as_str(), "binding:app");
        assert_eq!(ObjectId::connection(&local).as_str(), "connection:app");
        assert_eq!(
            ObjectId::property(&module, &version, &local).as_str(),
            "property:shell.desktop@1.0.0#app"
        );
        assert_eq!(
            ObjectId::slot(&module, &version, &local).as_str(),
            "slot:shell.desktop@1.0.0#app"
        );
        assert_eq!(
            ObjectId::port(&module, &version, &local).as_str(),
            "port:shell.desktop@1.0.0#app"
        );
        assert_eq!(
            ObjectId::dependency(&module, &version, &local).as_str(),
            "dependency:shell.desktop@1.0.0#app"
        );
        assert_eq!(
            ObjectId::file("composition/main.toml").as_str(),
            "file:composition/main.toml"
        );
    }

    #[test]
    fn severity_is_derived_from_the_code() {
        assert_eq!(
            Diagnostic::new(ErrorCode::RefModuleNotFound, "missing").severity,
            Severity::Error
        );
        assert_eq!(
            Diagnostic::new(ErrorCode::DependencyOptionalUnresolved, "optional").severity,
            Severity::Warning
        );
    }

    #[test]
    fn builders_attach_source_and_related_objects() {
        let range = SourceRange::new(Position::new(6, 1).unwrap(), Position::new(8, 14).unwrap());
        let diagnostic = Diagnostic::new(ErrorCode::RefModuleNotFound, "missing")
            .with_object_id(ObjectId::instance(&LocalId::parse("app").unwrap()))
            .with_source("composition/main.toml", range)
            .with_related(vec![ObjectId::project()])
            .with_hint("add the module definition");
        assert!(diagnostic.is_error());
        assert_eq!(diagnostic.file.as_deref(), Some("composition/main.toml"));
        assert_eq!(diagnostic.related, vec![ObjectId::project()]);
        assert_eq!(
            diagnostic.hint.as_deref(),
            Some("add the module definition")
        );
    }

    #[test]
    fn diagnostics_sort_by_file_range_code_and_object() {
        let mut diagnostics = [
            Diagnostic::new(ErrorCode::TomlSyntaxError, "b").with_source(
                "b.toml",
                Some(
                    SourceRange::new(Position::new(1, 1).unwrap(), Position::new(1, 2).unwrap())
                        .unwrap(),
                ),
            ),
            Diagnostic::new(ErrorCode::TomlSyntaxError, "a").with_source(
                "a.toml",
                Some(
                    SourceRange::new(Position::new(2, 1).unwrap(), Position::new(2, 2).unwrap())
                        .unwrap(),
                ),
            ),
            Diagnostic::new(ErrorCode::FileProjectMissing, "no file"),
        ];
        diagnostics.sort_by(Diagnostic::compare_order);
        assert_eq!(diagnostics[0].code, ErrorCode::FileProjectMissing);
        assert_eq!(diagnostics[1].file.as_deref(), Some("a.toml"));
        assert_eq!(diagnostics[2].file.as_deref(), Some("b.toml"));
    }
}
