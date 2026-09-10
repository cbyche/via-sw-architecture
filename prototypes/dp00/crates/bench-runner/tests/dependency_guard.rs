use std::collections::{HashMap, HashSet, VecDeque};
use std::process::Command;

use serde_json::Value;

#[test]
fn aut_crates_cannot_reach_bench_oracle() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(format!("{manifest_dir}/../../Cargo.toml"))
        .output()
        .expect("cargo metadata must run");
    assert!(output.status.success(), "cargo metadata failed");

    let metadata: Value = serde_json::from_slice(&output.stdout).expect("metadata is valid JSON");
    let packages = metadata["packages"].as_array().expect("packages array");
    let mut by_id = HashMap::new();
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();

    for package in packages {
        let id = package["id"].as_str().expect("package id").to_owned();
        by_id.insert(
            id,
            package["name"].as_str().expect("package name").to_owned(),
        );
    }

    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .expect("resolve nodes");
    for node in nodes {
        let id = node["id"].as_str().expect("node id").to_owned();
        let direct = node["dependencies"]
            .as_array()
            .expect("dependency array")
            .iter()
            .map(|dependency| dependency.as_str().expect("dependency id").to_owned())
            .collect();
        deps.insert(id, direct);
    }

    for (id, name) in &by_id {
        if !name.starts_with("alternative-") {
            continue;
        }
        let mut queue = VecDeque::from([id.clone()]);
        let mut visited = HashSet::new();
        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            assert_ne!(
                by_id.get(&current).map(String::as_str),
                Some("bench-oracle"),
                "{name} has a direct or transitive dependency on bench-oracle"
            );
            queue.extend(deps.get(&current).into_iter().flatten().cloned());
        }
    }
}

#[test]
fn evaluation_support_boundary_keeps_oracle_out_of_aut_visible_core() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let core_manifest = std::fs::read_to_string(format!("{manifest_dir}/../bench-core/Cargo.toml"))
        .expect("bench-core manifest is readable");
    assert!(!core_manifest.contains("bench-oracle"));
}
