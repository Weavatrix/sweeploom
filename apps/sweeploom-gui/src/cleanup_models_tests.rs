use super::*;
#[test]
fn model_names_aliases_and_digest_paths_are_checked() {
    let root = std::path::PathBuf::from("/models/manifests");
    assert_eq!(
        manifest_name(&root, &root.join("registry.ollama.ai/library/qwen3/8b")),
        Some("qwen3:8b".into())
    );
    assert_eq!(
        manifest_name(&root, &root.join("registry.ollama.ai/team/model/latest")),
        Some("team/model:latest".into())
    );
    assert!(!valid_digest("sha256:../../credentials"));
    let loaded = serde_json::json!({"models":[{"name":"alias:latest","digest":"same"}]});
    assert!(is_loaded(&loaded, "other:latest", "same"));
    assert!(!is_loaded(&loaded, "other:latest", "different"));
}

#[test]
fn loaded_changed_missing_or_unverifiable_models_are_never_deleted() {
    let item = Item {
        id: "model:tag".into(),
        name: "test".into(),
        kind: Kind::Model,
        bytes: None,
        usage: None,
        status: String::new(),
        scope: Some("original".into()),
        path: None,
        selected: true,
        enabled: true,
    };
    let tags = serde_json::json!({"models":[{"name":"model:tag","digest":"original"}]});
    let empty = serde_json::json!({"models":[]});
    assert!(validate_removal(&item, &tags, &empty).is_ok());
    assert!(
        validate_removal(
            &item,
            &tags,
            &serde_json::json!({"models":[{"name":"alias","digest":"original"}]})
        )
        .is_err()
    );
    assert!(
        validate_removal(
            &item,
            &serde_json::json!({"models":[{"name":"model:tag","digest":"replaced"}]}),
            &empty
        )
        .is_err()
    );
    assert!(validate_removal(&item, &empty, &empty).is_err());
    assert!(validate_removal(&item, &tags, &serde_json::json!({})).is_err());
}

#[cfg(unix)]
#[test]
fn offline_models_count_allocated_blobs_once_and_reject_digest_escape() {
    let root = std::env::temp_dir().join(format!("sweeploom-ollama-{}", std::process::id()));
    let digest = format!("sha256:{}", "a".repeat(64));
    let blob = root.join("blobs").join(digest.replace(':', "-"));
    fs::create_dir_all(blob.parent().unwrap()).unwrap();
    fs::write(&blob, b"data").unwrap();
    fs::File::options()
        .write(true)
        .open(&blob)
        .unwrap()
        .set_len(1_000_000_000)
        .unwrap();
    let manifest = root.join("manifests/registry.ollama.ai/library/model/latest");
    fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    fs::write(&manifest, serde_json::json!({"config":{"digest":digest},"layers":[{"digest":digest},{"digest":"sha256:../../secret"}]}).to_string()).unwrap();
    let models = offline(&root);
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model:latest");
    let usage = models[0].usage.unwrap();
    assert_eq!(usage.files, 1);
    assert_eq!(usage.logical_bytes, 1_000_000_000);
    assert!(usage.bytes < usage.logical_bytes);
    assert!(!usage.complete);
    assert!(!models[0].enabled);
    fs::remove_dir_all(root).unwrap();
}
