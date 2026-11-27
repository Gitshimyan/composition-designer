use std::collections::{BTreeMap, BTreeSet};

use composition_domain::{Binding, CompositionModel, Connection, Instance, ObjectId};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObjectChange {
    pub object_id: ObjectId,
    pub kind: ChangeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SourceDiff {
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelDiff {
    pub summary: DiffSummary,
    pub changes: Vec<ObjectChange>,
}

impl ModelDiff {
    pub fn empty() -> Self {
        Self {
            summary: DiffSummary::default(),
            changes: Vec::new(),
        }
    }

    /// 实例 / Binding / Connection 的结构化 Diff；对象值用规范化 JSON 表示。
    pub fn between(before: &CompositionModel, after: &CompositionModel) -> Self {
        let mut before_map = BTreeMap::new();
        let mut after_map = BTreeMap::new();
        collect(
            &mut before_map,
            &before.composition.instances,
            &before.composition.bindings,
        );
        collect(
            &mut after_map,
            &after.composition.instances,
            &after.composition.bindings,
        );
        collect_connections(&mut before_map, &before.composition.connections);
        collect_connections(&mut after_map, &after.composition.connections);

        let keys: BTreeSet<ObjectId> = before_map.keys().chain(after_map.keys()).cloned().collect();
        let mut changes = Vec::new();
        let mut summary = DiffSummary::default();
        for object_id in keys {
            match (before_map.get(&object_id), after_map.get(&object_id)) {
                (None, Some(after)) => {
                    summary.added += 1;
                    changes.push(ObjectChange {
                        object_id,
                        kind: ChangeKind::Added,
                        before: None,
                        after: Some(after.clone()),
                    });
                }
                (Some(before), None) => {
                    summary.removed += 1;
                    changes.push(ObjectChange {
                        object_id,
                        kind: ChangeKind::Removed,
                        before: Some(before.clone()),
                        after: None,
                    });
                }
                (Some(before), Some(after)) if before != after => {
                    summary.modified += 1;
                    changes.push(ObjectChange {
                        object_id,
                        kind: ChangeKind::Modified,
                        before: Some(before.clone()),
                        after: Some(after.clone()),
                    });
                }
                _ => {}
            }
        }
        Self { summary, changes }
    }
}

fn collect(map: &mut BTreeMap<ObjectId, Value>, instances: &[Instance], bindings: &[Binding]) {
    for instance in instances {
        map.insert(
            ObjectId::instance(&instance.id),
            serde_json::to_value(instance).expect("instance is serializable"),
        );
    }
    for binding in bindings {
        map.insert(
            ObjectId::binding(&binding.id),
            serde_json::to_value(binding).expect("binding is serializable"),
        );
    }
}

fn collect_connections(map: &mut BTreeMap<ObjectId, Value>, connections: &[Connection]) {
    for connection in connections {
        map.insert(
            ObjectId::connection(&connection.id),
            serde_json::to_value(connection).expect("connection is serializable"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_diff_has_no_changes() {
        let diff = ModelDiff::empty();
        assert_eq!(diff.summary, DiffSummary::default());
        assert!(diff.changes.is_empty());
    }
}
