use std::collections::BTreeMap;

use composition_domain::{ObjectId, SourceRef};

/// 模型对象到来源位置的映射。007 用它给诊断定位契约成员。
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    entries: BTreeMap<ObjectId, SourceRef>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, object: ObjectId, source: SourceRef) {
        self.entries.insert(object, source);
    }

    pub fn get(&self, object: &ObjectId) -> Option<&SourceRef> {
        self.entries.get(object)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
