use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use composition_core::validate_project;

fn write_project(instances: usize) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "composition-perf-{instances}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("composition")).unwrap();
    fs::create_dir_all(root.join("modules/shell")).unwrap();
    fs::create_dir_all(root.join("modules/item")).unwrap();

    fs::write(
        root.join("project.toml"),
        r#"[project]
schema_version = 1
id = "perf.project"
name = "Performance Project"

[workspace]
composition = "composition/main.toml"
module_roots = ["modules"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("modules/shell/module.toml"),
        r#"[module]
schema_version = 1
id = "perf.shell"
version = "1.0.0"
kind = "composite"

[[contract.slots]]
id = "items"
cardinality = "many"
required = true
accepts = ["perf.item"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("modules/item/module.toml"),
        r#"[module]
schema_version = 1
id = "perf.item"
version = "1.0.0"
kind = "primitive"

[contract.properties.title]
type = "string"
required = true
"#,
    )
    .unwrap();

    let mut composition = String::from(
        r#"[composition]
schema_version = 1
id = "perf.project"
root = "node-0"

[[instances]]
id = "node-0"
module = "perf.shell"
version = "1.0.0"
"#,
    );
    for index in 1..=instances {
        write!(
            composition,
            "\n[[instances]]\nid = \"node-{index}\"\nmodule = \"perf.item\"\nversion = \"1.0.0\"\n\n[instances.config]\ntitle = \"item-{index}\"\n"
        )
        .unwrap();
    }
    for index in 1..=instances {
        write!(
            composition,
            "\n[[bindings]]\nid = \"bind-{index}\"\nparent = \"node-0\"\nslot = \"items\"\nchild = \"node-{index}\"\n"
        )
        .unwrap();
    }
    fs::write(root.join("composition/main.toml"), composition).unwrap();
    root
}

#[test]
fn synthetic_large_project_validates_in_reasonable_time() {
    let instances = 80;
    let root = write_project(instances);
    let started = Instant::now();
    let validation = validate_project(&root);
    let elapsed = started.elapsed();
    assert!(validation.is_valid(), "{:?}", validation.diagnostics);
    let model = validation.model.expect("model");
    assert_eq!(model.composition.instances.len(), instances + 1);
    assert_eq!(model.composition.bindings.len(), instances);
    assert!(
        elapsed.as_secs_f64() < 5.0,
        "validation took {elapsed:?} for {} instances",
        instances + 1
    );
    let _ = fs::remove_dir_all(&root);
}
