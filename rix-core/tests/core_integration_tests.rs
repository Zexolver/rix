use std::fs;
use tempfile::TempDir;
use rix_core::{Package, RixContext, verify};

#[test]
fn test_end_to_end_package_lifecycle() {
    let tmp_dir = TempDir::new().unwrap();
    let ctx = RixContext::new(tmp_dir.path().to_path_buf());

    ctx.initialize_layout().unwrap();
    assert!(tmp_dir.path().join("groups/upstream").exists());

    let pkg1 = Package {
        name: "bat".to_string(),
        description: Some("Cat alternative with highlighting".to_string()),
        group: "default".to_string(),
        is_local_recipe: false,
    };
    ctx.add_package(pkg1).unwrap();

    let file_path = tmp_dir.path().join("groups/upstream/default.nix");
    assert!(file_path.exists());
    
    // Ensure our syntax check treats the fresh file as totally healthy
    assert!(verify::verify_nix_syntax(&file_path).is_ok());

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("pkgs.bat"));
}

#[test]
fn test_syntax_checker_catches_malformed_nix() {
    let tmp_dir = TempDir::new().unwrap();
    let file_path = tmp_dir.path().join("broken.nix");
    
    // Write broken, un-parseable trash text into the file
    fs::write(&file_path, "this { is completely broken syntax structure ===").unwrap();
    
    // Assert that our validation check detects it instead of letting it pass silently
    let check_result = verify::verify_nix_syntax(&file_path);
    assert!(check_result.is_err());
    
    if let Err(rix_core::RixError::InvalidNixSyntax(msg)) = check_result {
        assert!(msg.contains("syntax error"));
    } else {
        panic!("Expected InvalidNixSyntax error payload!");
    }
}
