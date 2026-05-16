use std::fs;
use tempfile::TempDir;
use rix_core::{Package, RixContext};

#[test]
fn test_end_to_end_package_lifecycle() {
    // 1. Initialize an isolated virtual sandbox directory
    let tmp_dir = TempDir::new().unwrap();
    let ctx = RixContext::new(tmp_dir.path().to_path_buf());

    ctx.initialize_layout().unwrap();
    assert!(tmp_dir.path().join("groups/upstream").exists());

    // 2. Validate clean installation injection and exact comment preservation
    let pkg1 = Package {
        name: "bat".to_string(),
        description: Some("Cat alternative with highlighting".to_string()),
        group: "default".to_string(),
        is_local_recipe: false,
    };
    ctx.add_package(pkg1).unwrap();

    let file_path = tmp_dir.path().join("groups/upstream/default.nix");
    assert!(file_path.exists());
    
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("pkgs.bat"));
    assert!(content.contains("# Cat alternative with highlighting"));

    // 3. Validate comment updates for existing targets (Idempotency and modification logic)
    let pkg1_updated = Package {
        name: "bat".to_string(),
        description: Some("Overwritten description string".to_string()),
        group: "default".to_string(),
        is_local_recipe: false,
    };
    ctx.add_package(pkg1_updated).unwrap();
    
    let content_updated = fs::read_to_string(&file_path).unwrap();
    assert!(content_updated.contains("# Overwritten description string"));
    assert!(!content_updated.contains("# Cat alternative with highlighting"));

    // 4. Validate complete listing inventory matrix extraction
    let list = ctx.list_all_packages().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].0, "bat");
    assert_eq!(list[0].1, "default");

    // 5. Validate package removal mechanics
    ctx.remove_package_from_file("bat", &file_path).unwrap();
    let content_after_removal = fs::read_to_string(&file_path).unwrap();
    assert!(!content_after_removal.contains("pkgs.bat"));
}
