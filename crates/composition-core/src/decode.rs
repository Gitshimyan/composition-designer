use std::collections::BTreeSet;
use std::ops::Range;

use composition_domain::{
    Binding, Composition, ConfigValue, Connection, Contract, DependencyRequirement, Diagnostic,
    ErrorCode, Instance, LocalId, ModuleDefinition, ModuleId, ModuleKind, ModuleRef, ObjectId,
    PortContract, PortDirection, PortRef, PropertyContract, PropertyType, SchemaVersion,
    SlotCardinality, SlotContract, TypeId, Version,
};
use composition_toml::{LoadedFile, LoadedWorkspace};
use toml_edit::{ArrayOfTables, Item, Table, Value};

use crate::source_map::SourceMap;

pub(crate) struct DecodedWorkspace {
    pub composition: Option<Composition>,
    pub modules: Vec<ModuleDefinition>,
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: SourceMap,
    pub invalid_instances: BTreeSet<LocalId>,
}

pub(crate) fn decode(loaded: &LoadedWorkspace) -> DecodedWorkspace {
    let mut diagnostics = Vec::new();
    let mut source_map = SourceMap::new();
    if let Some(project) = &loaded.project {
        if let Some(source) = &project.source {
            source_map.insert(ObjectId::project(), source.clone());
        }
    }

    let mut modules = Vec::new();
    for file in loaded.module_files() {
        if file.document().is_some() {
            if let Some(module) = decode_module(file, &mut diagnostics, &mut source_map) {
                modules.push(module);
            }
        }
    }

    let mut composition = None;
    let mut invalid_instances = BTreeSet::new();
    if loaded.project.is_some() {
        if let Some(file) = loaded.composition_file() {
            if file.document().is_some() {
                let (decoded, invalid) =
                    decode_composition(file, &mut diagnostics, &mut source_map);
                composition = decoded;
                invalid_instances = invalid;
            }
        }
    }

    DecodedWorkspace {
        composition,
        modules,
        diagnostics,
        source_map,
        invalid_instances,
    }
}

struct Ctx<'a> {
    file: &'a LoadedFile,
    diagnostics: &'a mut Vec<Diagnostic>,
    source_map: &'a mut SourceMap,
}

impl Ctx<'_> {
    fn push(
        &mut self,
        code: ErrorCode,
        message: impl AsRef<str>,
        span: Option<Range<usize>>,
        object: Option<&ObjectId>,
    ) {
        let mut diagnostic = Diagnostic::new(code, message.as_ref())
            .with_source(self.file.relative_path(), self.file.range(span));
        if let Some(object) = object {
            diagnostic = diagnostic.with_object_id(object.clone());
        }
        self.diagnostics.push(diagnostic);
    }

    fn map(&mut self, object: ObjectId, span: Option<Range<usize>>) {
        self.source_map.insert(object, self.file.source_ref(span));
    }
}

fn check_unknown(
    ctx: &mut Ctx<'_>,
    table: &Table,
    allowed: &[&str],
    context: &str,
    object: Option<&ObjectId>,
) {
    for (key, item) in table.iter() {
        if !allowed.contains(&key) {
            ctx.push(
                ErrorCode::SchemaUnknownField,
                format!("{context}: unknown field {key:?}"),
                item.span(),
                object,
            );
        }
    }
}

fn required_table<'a>(
    ctx: &mut Ctx<'_>,
    table: &'a Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<&'a Table> {
    match table.get(key) {
        None => {
            ctx.push(
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_table() {
            Some(table) => Some(table),
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a table"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn optional_table<'a>(
    ctx: &mut Ctx<'_>,
    table: &'a Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<&'a Table> {
    match table.get(key) {
        None => None,
        Some(item) => match item.as_table() {
            Some(table) => Some(table),
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a table"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn required_string(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<String> {
    match table.get(key) {
        None => {
            ctx.push(
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_str() {
            Some(value) => Some(value.to_owned()),
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a string"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn optional_string(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<String> {
    match table.get(key) {
        None => None,
        Some(item) => match item.as_str() {
            Some(value) => Some(value.to_owned()),
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a string"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn required_integer(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<i64> {
    match table.get(key) {
        None => {
            ctx.push(
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_integer() {
            Some(value) => Some(value),
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be an integer"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn required_bool(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<bool> {
    match table.get(key) {
        None => {
            ctx.push(
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_bool() {
            Some(value) => Some(value),
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a boolean"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn optional_bool(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
    default: bool,
) -> bool {
    match table.get(key) {
        None => default,
        Some(item) => match item.as_bool() {
            Some(value) => value,
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a boolean"),
                    item.span(),
                    object,
                );
                default
            }
        },
    }
}

fn optional_u64(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<u64> {
    match table.get(key) {
        None => None,
        Some(item) => match item.as_integer() {
            Some(value) if value >= 0 => u64::try_from(value).ok(),
            Some(_) => {
                ctx.push(
                    ErrorCode::SchemaInvalidValue,
                    format!("{context}.{key} must not be negative"),
                    item.span(),
                    object,
                );
                None
            }
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be an integer"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn optional_number(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<ConfigValue> {
    match table.get(key) {
        None => None,
        Some(item) => match item.as_value() {
            Some(value) => {
                if let Some(number) = value.as_integer() {
                    Some(ConfigValue::Integer(number))
                } else if let Some(number) = value.as_float() {
                    Some(ConfigValue::Float(number))
                } else {
                    ctx.push(
                        ErrorCode::SchemaTypeMismatch,
                        format!("{context}.{key} must be a number"),
                        item.span(),
                        object,
                    );
                    None
                }
            }
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a number"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn required_string_array(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<Vec<String>> {
    match table.get(key) {
        None => {
            ctx.push(
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                table.span(),
                object,
            );
            None
        }
        Some(item) => string_array(ctx, item, context, key, object),
    }
}

fn optional_string_array(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<Vec<String>> {
    table
        .get(key)
        .and_then(|item| string_array(ctx, item, context, key, object))
}

fn string_array(
    ctx: &mut Ctx<'_>,
    item: &Item,
    context: &str,
    key: &str,
    object: Option<&ObjectId>,
) -> Option<Vec<String>> {
    match item.as_array() {
        Some(array) => {
            let mut values = Vec::new();
            let mut valid = true;
            for value in array.iter() {
                match value.as_str() {
                    Some(text) => values.push(text.to_owned()),
                    None => {
                        ctx.push(
                            ErrorCode::SchemaTypeMismatch,
                            format!("{context}.{key} entries must be strings"),
                            value.span(),
                            object,
                        );
                        valid = false;
                    }
                }
            }
            if valid {
                Some(values)
            } else {
                None
            }
        }
        None => {
            ctx.push(
                ErrorCode::SchemaTypeMismatch,
                format!("{context}.{key} must be an array"),
                item.span(),
                object,
            );
            None
        }
    }
}

fn optional_array_tables<'a>(
    ctx: &mut Ctx<'_>,
    table: &'a Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<&'a ArrayOfTables> {
    match table.get(key) {
        None => None,
        Some(item) => match item.as_array_of_tables() {
            Some(array) => Some(array),
            None => {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be an array of tables"),
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn decode_config_value(
    ctx: &mut Ctx<'_>,
    value: &Value,
    object: Option<&ObjectId>,
) -> Option<ConfigValue> {
    if let Some(text) = value.as_str() {
        return Some(ConfigValue::String(text.to_owned()));
    }
    if let Some(integer) = value.as_integer() {
        return Some(ConfigValue::Integer(integer));
    }
    if let Some(float) = value.as_float() {
        return Some(ConfigValue::Float(float));
    }
    if let Some(boolean) = value.as_bool() {
        return Some(ConfigValue::Boolean(boolean));
    }
    if let Some(array) = value.as_array() {
        let mut values = Vec::new();
        for item in array.iter() {
            let value = decode_config_value(ctx, item, object)?;
            values.push(value);
        }
        return Some(ConfigValue::Array(values));
    }
    ctx.push(
        ErrorCode::SchemaUnsupportedFeature,
        "unsupported config value (table or datetime); v0.1 supports scalars and arrays",
        value.span(),
        object,
    );
    None
}

fn decode_module(
    file: &LoadedFile,
    diagnostics: &mut Vec<Diagnostic>,
    source_map: &mut SourceMap,
) -> Option<ModuleDefinition> {
    let document = file.document()?;
    let root = document.as_table();
    let mut ctx = Ctx {
        file,
        diagnostics,
        source_map,
    };
    check_unknown(
        &mut ctx,
        root,
        &["module", "description", "contract"],
        "module file",
        None,
    );

    let module_item = match root.get("module") {
        Some(item) => item,
        None => {
            ctx.push(
                ErrorCode::SchemaMissingField,
                "module: [module] table is required",
                root.span(),
                None,
            );
            return None;
        }
    };
    let module_table = match module_item.as_table() {
        Some(table) => table,
        None => {
            ctx.push(
                ErrorCode::SchemaTypeMismatch,
                "module: [module] must be a table",
                module_item.span(),
                None,
            );
            return None;
        }
    };
    check_unknown(
        &mut ctx,
        module_table,
        &["schema_version", "id", "version", "kind"],
        "module",
        None,
    );

    let schema_version =
        required_integer(&mut ctx, module_table, "schema_version", "module", None)?;
    let schema_version = SchemaVersion::new(u16::try_from(schema_version).unwrap_or(0));
    if !schema_version.is_supported() {
        ctx.push(
            ErrorCode::SchemaUnsupportedVersion,
            "module.schema_version must be 1 for v0.1",
            module_table.get("schema_version").and_then(Item::span),
            None,
        );
    }
    let id_raw = required_string(&mut ctx, module_table, "id", "module", None)?;
    let id = match ModuleId::parse(&id_raw) {
        Ok(id) => id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                module_table.get("id").and_then(Item::span),
                None,
            );
            return None;
        }
    };
    let version_raw = required_string(&mut ctx, module_table, "version", "module", None)?;
    let version = match Version::parse(&version_raw) {
        Ok(version) => version,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                module_table.get("version").and_then(Item::span),
                None,
            );
            return None;
        }
    };
    let kind_raw = required_string(&mut ctx, module_table, "kind", "module", None)?;
    let kind = match ModuleKind::parse(&kind_raw) {
        Ok(kind) => kind,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                module_table.get("kind").and_then(Item::span),
                None,
            );
            return None;
        }
    };
    let module_object = ObjectId::module(&id, &version);
    ctx.map(module_object.clone(), module_item.span());

    let mut description = None;
    if let Some(description_table) = optional_table(
        &mut ctx,
        root,
        "description",
        "module file",
        Some(&module_object),
    ) {
        check_unknown(
            &mut ctx,
            description_table,
            &["summary", "details"],
            "description",
            Some(&module_object),
        );
        if let Some(summary) = required_string(
            &mut ctx,
            description_table,
            "summary",
            "description",
            Some(&module_object),
        ) {
            if summary.trim().is_empty() {
                ctx.push(
                    ErrorCode::SchemaEmptyValue,
                    "description.summary must not be empty",
                    description_table.get("summary").and_then(Item::span),
                    Some(&module_object),
                );
            }
            description = Some(summary);
        }
        let _ = optional_string(
            &mut ctx,
            description_table,
            "details",
            "description",
            Some(&module_object),
        );
    }

    let mut contract = Contract::new();
    if let Some(contract_table) = optional_table(
        &mut ctx,
        root,
        "contract",
        "module file",
        Some(&module_object),
    ) {
        contract = decode_contract(&mut ctx, contract_table, &id, &version, &module_object);
    }

    let mut module = ModuleDefinition::new(id, version, kind, contract);
    module.description = description;
    module.source = Some(file.source_ref(module_item.span()));
    Some(module)
}

fn decode_contract(
    ctx: &mut Ctx<'_>,
    table: &Table,
    module_id: &ModuleId,
    version: &Version,
    module_object: &ObjectId,
) -> Contract {
    check_unknown(
        ctx,
        table,
        &["properties", "slots", "ports", "dependencies"],
        "contract",
        Some(module_object),
    );
    let mut contract = Contract::new();

    if let Some(properties) =
        optional_table(ctx, table, "properties", "contract", Some(module_object))
    {
        for (key, item) in properties.iter() {
            let property_id = match LocalId::parse(key) {
                Ok(id) => id,
                Err(error) => {
                    ctx.push(
                        error.code(),
                        error.message(),
                        item.span(),
                        Some(module_object),
                    );
                    continue;
                }
            };
            let property_object = ObjectId::property(module_id, version, &property_id);
            ctx.map(property_object.clone(), item.span());
            let Some(property_table) = item.as_table() else {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    format!("property {property_id:?} must be a table"),
                    item.span(),
                    Some(&property_object),
                );
                continue;
            };
            if let Some(property) = decode_property(ctx, property_table, &property_object, true) {
                if let Err(error) = contract.insert_property(property_id, property) {
                    ctx.push(
                        error.code(),
                        error.message(),
                        property_table.span(),
                        Some(&property_object),
                    );
                }
            }
        }
    }

    if let Some(slots) = optional_array_tables(ctx, table, "slots", "contract", Some(module_object))
    {
        for slot_table in slots.iter() {
            if let Some(slot) = decode_slot(ctx, slot_table, module_id, version, module_object) {
                let id = slot.0;
                let contract_value = slot.1;
                if let Err(error) = contract.insert_slot(id, contract_value) {
                    ctx.push(
                        error.code(),
                        error.message(),
                        slot_table.span(),
                        Some(module_object),
                    );
                }
            }
        }
    }

    if let Some(ports) = optional_array_tables(ctx, table, "ports", "contract", Some(module_object))
    {
        for port_table in ports.iter() {
            if let Some(port) = decode_port(ctx, port_table, module_id, version, module_object) {
                let id = port.0;
                let contract_value = port.1;
                if let Err(error) = contract.insert_port(id, contract_value) {
                    ctx.push(
                        error.code(),
                        error.message(),
                        port_table.span(),
                        Some(module_object),
                    );
                }
            }
        }
    }

    if let Some(dependencies) =
        optional_array_tables(ctx, table, "dependencies", "contract", Some(module_object))
    {
        for dependency_table in dependencies.iter() {
            if let Some(dependency) =
                decode_dependency(ctx, dependency_table, module_id, version, module_object)
            {
                let id = dependency.0;
                let contract_value = dependency.1;
                if let Err(error) = contract.insert_dependency(id, contract_value) {
                    ctx.push(
                        error.code(),
                        error.message(),
                        dependency_table.span(),
                        Some(module_object),
                    );
                }
            }
        }
    }

    contract
}

fn decode_property(
    ctx: &mut Ctx<'_>,
    table: &Table,
    object: &ObjectId,
    allow_items: bool,
) -> Option<PropertyContract> {
    let context = "property";
    check_unknown(
        ctx,
        table,
        &[
            "type",
            "required",
            "default",
            "description",
            "values",
            "min_length",
            "max_length",
            "min",
            "max",
            "min_items",
            "max_items",
            "items",
        ],
        context,
        Some(object),
    );
    let type_raw = required_string(ctx, table, "type", context, Some(object))?;
    let property_type = match PropertyType::parse(&type_raw) {
        Ok(property_type) => property_type,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("type").and_then(Item::span),
                Some(object),
            );
            return None;
        }
    };
    let mut property = PropertyContract::new(property_type, false);
    property.required = optional_bool(ctx, table, "required", context, Some(object), false);

    if let Some(item) = table.get("default") {
        match item.as_value() {
            Some(value) => {
                if let Some(default) = decode_config_value(ctx, value, Some(object)) {
                    property.default = Some(default);
                }
            }
            None => ctx.push(
                ErrorCode::SchemaUnsupportedFeature,
                "property.default must be a scalar or array",
                item.span(),
                Some(object),
            ),
        }
    }
    property.description = optional_string(ctx, table, "description", context, Some(object));
    property.enum_values =
        optional_string_array(ctx, table, "values", context, Some(object)).unwrap_or_default();
    property.min_length = optional_u64(ctx, table, "min_length", context, Some(object));
    property.max_length = optional_u64(ctx, table, "max_length", context, Some(object));
    property.min = optional_number(ctx, table, "min", context, Some(object));
    property.max = optional_number(ctx, table, "max", context, Some(object));
    property.min_items = optional_u64(ctx, table, "min_items", context, Some(object));
    property.max_items = optional_u64(ctx, table, "max_items", context, Some(object));

    if allow_items {
        if let Some(item) = table.get("items") {
            match item.as_table() {
                Some(items_table) => {
                    property.items = decode_item_property(ctx, items_table, object).map(Box::new);
                }
                None => ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    "property.items must be a table",
                    item.span(),
                    Some(object),
                ),
            }
        }
    } else if table.get("items").is_some() {
        ctx.push(
            ErrorCode::SchemaUnsupportedFeature,
            "nested array item contracts are not supported in v0.1",
            table.get("items").and_then(Item::span),
            Some(object),
        );
    }

    Some(property)
}

fn decode_item_property(
    ctx: &mut Ctx<'_>,
    table: &Table,
    object: &ObjectId,
) -> Option<PropertyContract> {
    let context = "property.items";
    check_unknown(
        ctx,
        table,
        &["type", "values", "min_length", "max_length", "min", "max"],
        context,
        Some(object),
    );
    let type_raw = required_string(ctx, table, "type", context, Some(object))?;
    let property_type = match PropertyType::parse(&type_raw) {
        Ok(property_type) => property_type,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("type").and_then(Item::span),
                Some(object),
            );
            return None;
        }
    };
    let mut property = PropertyContract::new(property_type, false);
    property.enum_values =
        optional_string_array(ctx, table, "values", context, Some(object)).unwrap_or_default();
    property.min_length = optional_u64(ctx, table, "min_length", context, Some(object));
    property.max_length = optional_u64(ctx, table, "max_length", context, Some(object));
    property.min = optional_number(ctx, table, "min", context, Some(object));
    property.max = optional_number(ctx, table, "max", context, Some(object));
    Some(property)
}

fn decode_slot(
    ctx: &mut Ctx<'_>,
    table: &Table,
    module_id: &ModuleId,
    version: &Version,
    module_object: &ObjectId,
) -> Option<(LocalId, SlotContract)> {
    let context = "slot";
    check_unknown(
        ctx,
        table,
        &["id", "cardinality", "required", "accepts"],
        context,
        Some(module_object),
    );
    let id_raw = required_string(ctx, table, "id", context, Some(module_object))?;
    let id = match LocalId::parse(&id_raw) {
        Ok(id) => id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("id").and_then(Item::span),
                Some(module_object),
            );
            return None;
        }
    };
    let object = ObjectId::slot(module_id, version, &id);
    ctx.map(object.clone(), table.span());
    let cardinality_raw = required_string(ctx, table, "cardinality", context, Some(&object))?;
    let cardinality = match SlotCardinality::parse(&cardinality_raw) {
        Ok(cardinality) => cardinality,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("cardinality").and_then(Item::span),
                Some(&object),
            );
            return None;
        }
    };
    let required = required_bool(ctx, table, "required", context, Some(&object))?;
    let accepts_raw = required_string_array(ctx, table, "accepts", context, Some(&object))?;
    let mut accepts = Vec::new();
    for raw in accepts_raw {
        match ModuleId::parse(&raw) {
            Ok(module) => accepts.push(module),
            Err(error) => ctx.push(
                error.code(),
                error.message(),
                table.get("accepts").and_then(Item::span),
                Some(&object),
            ),
        }
    }
    Some((id, SlotContract::new(cardinality, required, accepts)))
}

fn decode_port(
    ctx: &mut Ctx<'_>,
    table: &Table,
    module_id: &ModuleId,
    version: &Version,
    module_object: &ObjectId,
) -> Option<(LocalId, PortContract)> {
    let context = "port";
    check_unknown(
        ctx,
        table,
        &["id", "direction", "type", "multi", "protocol"],
        context,
        Some(module_object),
    );
    let id_raw = required_string(ctx, table, "id", context, Some(module_object))?;
    let id = match LocalId::parse(&id_raw) {
        Ok(id) => id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("id").and_then(Item::span),
                Some(module_object),
            );
            return None;
        }
    };
    let object = ObjectId::port(module_id, version, &id);
    ctx.map(object.clone(), table.span());
    let direction_raw = required_string(ctx, table, "direction", context, Some(&object))?;
    let direction = match PortDirection::parse(&direction_raw) {
        Ok(direction) => direction,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("direction").and_then(Item::span),
                Some(&object),
            );
            return None;
        }
    };
    let type_raw = required_string(ctx, table, "type", context, Some(&object))?;
    let type_id = match TypeId::parse(&type_raw) {
        Ok(type_id) => type_id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("type").and_then(Item::span),
                Some(&object),
            );
            return None;
        }
    };
    let mut port = PortContract::new(direction, type_id);
    port.multi = optional_bool(ctx, table, "multi", context, Some(&object), false);
    port.protocol = optional_string(ctx, table, "protocol", context, Some(&object));
    Some((id, port))
}

fn decode_dependency(
    ctx: &mut Ctx<'_>,
    table: &Table,
    module_id: &ModuleId,
    version: &Version,
    module_object: &ObjectId,
) -> Option<(LocalId, DependencyRequirement)> {
    let context = "dependency";
    check_unknown(
        ctx,
        table,
        &["id", "module", "version", "optional"],
        context,
        Some(module_object),
    );
    let id_raw = required_string(ctx, table, "id", context, Some(module_object))?;
    let id = match LocalId::parse(&id_raw) {
        Ok(id) => id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("id").and_then(Item::span),
                Some(module_object),
            );
            return None;
        }
    };
    let object = ObjectId::dependency(module_id, version, &id);
    ctx.map(object.clone(), table.span());
    let target_raw = required_string(ctx, table, "module", context, Some(&object))?;
    let target = match ModuleId::parse(&target_raw) {
        Ok(target) => target,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("module").and_then(Item::span),
                Some(&object),
            );
            return None;
        }
    };
    let version = match optional_string(ctx, table, "version", context, Some(&object)) {
        Some(raw) => match Version::parse(&raw) {
            Ok(version) => Some(version),
            Err(error) => {
                ctx.push(
                    error.code(),
                    error.message(),
                    table.get("version").and_then(Item::span),
                    Some(&object),
                );
                return None;
            }
        },
        None => None,
    };
    let optional = optional_bool(ctx, table, "optional", context, Some(&object), false);
    Some((id, DependencyRequirement::new(target, version, optional)))
}

fn decode_composition(
    file: &LoadedFile,
    diagnostics: &mut Vec<Diagnostic>,
    source_map: &mut SourceMap,
) -> (Option<Composition>, BTreeSet<LocalId>) {
    let mut invalid_instances = BTreeSet::new();
    let Some(document) = file.document() else {
        return (None, invalid_instances);
    };
    let root = document.as_table();
    let mut ctx = Ctx {
        file,
        diagnostics,
        source_map,
    };
    check_unknown(
        &mut ctx,
        root,
        &["composition", "instances", "bindings", "connections"],
        "composition file",
        Some(&ObjectId::composition()),
    );

    let composition_item = match root.get("composition") {
        Some(item) => item,
        None => {
            ctx.push(
                ErrorCode::SchemaMissingField,
                "composition: [composition] table is required",
                root.span(),
                Some(&ObjectId::composition()),
            );
            return (None, invalid_instances);
        }
    };
    let composition_table = match composition_item.as_table() {
        Some(table) => table,
        None => {
            ctx.push(
                ErrorCode::SchemaTypeMismatch,
                "composition: [composition] must be a table",
                composition_item.span(),
                Some(&ObjectId::composition()),
            );
            return (None, invalid_instances);
        }
    };
    let composition_object = ObjectId::composition();
    ctx.map(composition_object.clone(), composition_item.span());
    check_unknown(
        &mut ctx,
        composition_table,
        &["schema_version", "id", "root"],
        "composition",
        Some(&composition_object),
    );

    let schema_version = required_integer(
        &mut ctx,
        composition_table,
        "schema_version",
        "composition",
        Some(&composition_object),
    )
    .unwrap_or(1);
    let schema_version = SchemaVersion::new(u16::try_from(schema_version).unwrap_or(0));
    if !schema_version.is_supported() {
        ctx.push(
            ErrorCode::SchemaUnsupportedVersion,
            "composition.schema_version must be 1 for v0.1",
            composition_table.get("schema_version").and_then(Item::span),
            Some(&composition_object),
        );
    }
    let id_raw = match required_string(
        &mut ctx,
        composition_table,
        "id",
        "composition",
        Some(&composition_object),
    ) {
        Some(value) => value,
        None => return (None, invalid_instances),
    };
    let id = match ModuleId::parse(&id_raw) {
        Ok(id) => id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                composition_table.get("id").and_then(Item::span),
                Some(&composition_object),
            );
            return (None, invalid_instances);
        }
    };
    let root_raw = match required_string(
        &mut ctx,
        composition_table,
        "root",
        "composition",
        Some(&composition_object),
    ) {
        Some(value) => value,
        None => return (None, invalid_instances),
    };
    let root_id = match LocalId::parse(&root_raw) {
        Ok(id) => id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                composition_table.get("root").and_then(Item::span),
                Some(&composition_object),
            );
            return (None, invalid_instances);
        }
    };

    let mut composition = Composition::new(schema_version, id, root_id);
    composition.source = Some(file.source_ref(composition_item.span()));

    if let Some(instances) = optional_array_tables(
        &mut ctx,
        root,
        "instances",
        "composition file",
        Some(&composition_object),
    ) {
        let mut seen = BTreeSet::new();
        for instance_table in instances.iter() {
            let object = decode_instance(&mut ctx, instance_table, &mut invalid_instances);
            if let Some(instance) = object {
                if !seen.insert(instance.id.clone()) {
                    ctx.push(
                        ErrorCode::IdDuplicateInstance,
                        format!("duplicate instance id {:?}", instance.id.as_str()),
                        instance_table.span(),
                        Some(&ObjectId::instance(&instance.id)),
                    );
                    continue;
                }
                composition.instances.push(instance);
            }
        }
    }

    if let Some(bindings) = optional_array_tables(
        &mut ctx,
        root,
        "bindings",
        "composition file",
        Some(&composition_object),
    ) {
        let mut seen = BTreeSet::new();
        for binding_table in bindings.iter() {
            if let Some(binding) = decode_binding(&mut ctx, binding_table) {
                if !seen.insert(binding.id.clone()) {
                    ctx.push(
                        ErrorCode::IdDuplicateBinding,
                        format!("duplicate binding id {:?}", binding.id.as_str()),
                        binding_table.span(),
                        Some(&ObjectId::binding(&binding.id)),
                    );
                    continue;
                }
                composition.bindings.push(binding);
            }
        }
    }

    if let Some(connections) = optional_array_tables(
        &mut ctx,
        root,
        "connections",
        "composition file",
        Some(&composition_object),
    ) {
        let mut seen = BTreeSet::new();
        for connection_table in connections.iter() {
            if let Some(connection) = decode_connection(&mut ctx, connection_table) {
                if !seen.insert(connection.id.clone()) {
                    ctx.push(
                        ErrorCode::IdDuplicateConnection,
                        format!("duplicate connection id {:?}", connection.id.as_str()),
                        connection_table.span(),
                        Some(&ObjectId::connection(&connection.id)),
                    );
                    continue;
                }
                composition.connections.push(connection);
            }
        }
    }
    (Some(composition), invalid_instances)
}

fn decode_instance(
    ctx: &mut Ctx<'_>,
    table: &Table,
    invalid_instances: &mut BTreeSet<LocalId>,
) -> Option<Instance> {
    let context = "instance";
    check_unknown(
        ctx,
        table,
        &["id", "module", "version", "config", "dependencies"],
        context,
        None,
    );
    let id_raw = required_string(ctx, table, "id", context, None)?;
    let id = match LocalId::parse(&id_raw) {
        Ok(id) => id,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("id").and_then(Item::span),
                None,
            );
            return None;
        }
    };
    let object = ObjectId::instance(&id);
    ctx.map(object.clone(), table.span());

    let module_raw = required_string(ctx, table, "module", context, Some(&object));
    let version_raw = required_string(ctx, table, "version", context, Some(&object));
    let (Some(module_raw), Some(version_raw)) = (module_raw, version_raw) else {
        invalid_instances.insert(id);
        return None;
    };
    let module = match ModuleId::parse(&module_raw) {
        Ok(module) => module,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("module").and_then(Item::span),
                Some(&object),
            );
            invalid_instances.insert(id);
            return None;
        }
    };
    let version = match Version::parse(&version_raw) {
        Ok(version) => version,
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get("version").and_then(Item::span),
                Some(&object),
            );
            invalid_instances.insert(id);
            return None;
        }
    };

    let mut instance = Instance::new(id, ModuleRef::new(module, version));
    if let Some(config_table) = optional_table(ctx, table, "config", context, Some(&object)) {
        for (key, item) in config_table.iter() {
            let property_id = match LocalId::parse(key) {
                Ok(id) => id,
                Err(error) => {
                    ctx.push(error.code(), error.message(), item.span(), Some(&object));
                    continue;
                }
            };
            let Some(value) = item.as_value() else {
                ctx.push(
                    ErrorCode::SchemaUnsupportedFeature,
                    "instance config values must be scalars or arrays",
                    item.span(),
                    Some(&object),
                );
                continue;
            };
            if let Some(value) = decode_config_value(ctx, value, Some(&object)) {
                instance.config.insert(property_id, value);
            }
        }
    }
    if let Some(dependencies_table) =
        optional_table(ctx, table, "dependencies", context, Some(&object))
    {
        for (key, item) in dependencies_table.iter() {
            let requirement = match LocalId::parse(key) {
                Ok(id) => id,
                Err(error) => {
                    ctx.push(error.code(), error.message(), item.span(), Some(&object));
                    continue;
                }
            };
            let Some(value) = item.as_str() else {
                ctx.push(
                    ErrorCode::SchemaTypeMismatch,
                    "instance dependency provider must be a string",
                    item.span(),
                    Some(&object),
                );
                continue;
            };
            let provider = match LocalId::parse(value) {
                Ok(provider) => provider,
                Err(error) => {
                    ctx.push(error.code(), error.message(), item.span(), Some(&object));
                    continue;
                }
            };
            instance.dependency_providers.insert(requirement, provider);
        }
    }

    instance.source = Some(ctx.file.source_ref(table.span()));
    Some(instance)
}

fn decode_binding(ctx: &mut Ctx<'_>, table: &Table) -> Option<Binding> {
    let context = "binding";
    check_unknown(
        ctx,
        table,
        &["id", "parent", "slot", "child"],
        context,
        None,
    );
    let id = local_required(ctx, table, "id", context, None)?;
    let object = ObjectId::binding(&id);
    ctx.map(object.clone(), table.span());
    let parent = local_required(ctx, table, "parent", context, Some(&object))?;
    let slot = local_required(ctx, table, "slot", context, Some(&object))?;
    let child = local_required(ctx, table, "child", context, Some(&object))?;
    let mut binding = Binding::new(id, parent, slot, child);
    binding.source = Some(ctx.file.source_ref(table.span()));
    Some(binding)
}

fn decode_connection(ctx: &mut Ctx<'_>, table: &Table) -> Option<Connection> {
    let context = "connection";
    check_unknown(ctx, table, &["id", "from", "to"], context, None);
    let id = local_required(ctx, table, "id", context, None)?;
    let object = ObjectId::connection(&id);
    ctx.map(object.clone(), table.span());
    let from_table = required_table(ctx, table, "from", context, Some(&object))?;
    check_unknown(
        ctx,
        from_table,
        &["instance", "port"],
        "connection.from",
        Some(&object),
    );
    let from = PortRef::new(
        local_required(
            ctx,
            from_table,
            "instance",
            "connection.from",
            Some(&object),
        )?,
        local_required(ctx, from_table, "port", "connection.from", Some(&object))?,
    );
    let to_table = required_table(ctx, table, "to", context, Some(&object))?;
    check_unknown(
        ctx,
        to_table,
        &["instance", "port"],
        "connection.to",
        Some(&object),
    );
    let to = PortRef::new(
        local_required(ctx, to_table, "instance", "connection.to", Some(&object))?,
        local_required(ctx, to_table, "port", "connection.to", Some(&object))?,
    );
    let mut connection = Connection::new(id, from, to);
    connection.source = Some(ctx.file.source_ref(table.span()));
    Some(connection)
}

fn local_required(
    ctx: &mut Ctx<'_>,
    table: &Table,
    key: &str,
    context: &str,
    object: Option<&ObjectId>,
) -> Option<LocalId> {
    let raw = required_string(ctx, table, key, context, object)?;
    match LocalId::parse(&raw) {
        Ok(id) => Some(id),
        Err(error) => {
            ctx.push(
                error.code(),
                error.message(),
                table.get(key).and_then(Item::span),
                object,
            );
            None
        }
    }
}
