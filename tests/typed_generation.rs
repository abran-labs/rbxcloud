use std::process::Command;

#[test]
fn generated_sources_are_reproducible() {
    let status = Command::new("python3")
        .args(["scripts/gen_typed_api.py", "--check"])
        .status()
        .expect("python3 is required for typed API reproducibility checks");
    assert!(status.success());
}

#[test]
fn every_storage_operation_has_an_explicit_method_and_command() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../openapi/coverage.json")).unwrap();
    let sdk = include_str!("../src/rbx/generated/storage.rs");
    let cli = include_str!("../src/cli/generated/storage.rs");
    let mut count = 0;
    for operation in manifest["operations"].as_array().unwrap() {
        let family = operation["family"].as_str().unwrap();
        if !["data_stores", "ordered_data_stores", "memory_stores"].contains(&family) {
            continue;
        }
        count += 1;
        assert_eq!(operation["status"], "implemented");
        assert!(
            sdk.contains(operation["path"].as_str().unwrap()),
            "missing fixed SDK path for {}",
            operation["id"]
        );
        let method = operation["cli"]
            .as_str()
            .unwrap()
            .split(' ')
            .next_back()
            .unwrap()
            .replace('-', "_");
        assert!(
            sdk.contains(&format!("pub async fn {method}")),
            "missing typed SDK method for {}",
            operation["id"]
        );
        assert!(cli.contains(&format!(
            "pub struct {}",
            command_struct(operation["cli"].as_str().unwrap())
        )));
    }
    assert_eq!(count, 34);
    assert!(!cli.contains("pub url:"));
    assert!(!cli.contains("pub method:"));
    assert!(!cli.contains("pub body:"));
}

fn command_struct(cli: &str) -> String {
    cli.split(' ')
        .nth(1)
        .unwrap()
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
        })
        .collect()
}
