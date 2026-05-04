use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_cli_boards_json_output() {
    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.arg("boards")
       .arg("--json-output");

    // Output should succeed and contain JSON format
    cmd.assert()
       .success()
       .stdout(predicates::str::contains("\"id\": \"uno\""))
       .stdout(predicates::str::contains("\"mcu\": \"ATMEGA328P\""));
}

#[test]
fn test_cli_project_init_pure_mode() {
    // Create a temporary directory that deletes itself after the test
    let temp = tempdir().unwrap();
    let project_name = "test_pure_blink";

    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.current_dir(&temp)
       .arg("project")
       .arg("init")
       .arg(project_name)
       .arg("--board")
       .arg("uno");

    // We assume init succeeds (requires cargo installed on the testing machine)
    cmd.assert().success();

    let proj_path = temp.path().join(project_name);
    
    // Check if the file structure was created
    assert!(proj_path.exists());
    assert!(proj_path.join("Cargo.toml").exists());
    assert!(proj_path.join("Prustio.toml").exists());
    assert!(proj_path.join(".cargo/config.toml").exists());
    assert!(proj_path.join("rust-toolchain.toml").exists());
    assert!(proj_path.join("src/main.rs").exists());

    // Verify Prustio.toml contents
    let prustio_toml = std::fs::read_to_string(proj_path.join("Prustio.toml")).unwrap();
    assert!(prustio_toml.contains("hybrid_mode = false"));
}

#[test]
fn test_cli_project_init_hybrid_mode() {
    let temp = tempdir().unwrap();
    let project_name = "test_hybrid_blink";

    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.current_dir(&temp)
       .arg("project")
       .arg("init")
       .arg(project_name)
       .arg("--board")
       .arg("nanoatmega328")
       .arg("--hybrid"); // Trigger hybrid mode

    cmd.assert().success();

    let proj_path = temp.path().join(project_name);
    
    let prustio_toml = std::fs::read_to_string(proj_path.join("Prustio.toml")).unwrap();
    assert!(prustio_toml.contains("hybrid_mode = true"));
    assert!(prustio_toml.contains("[env.nanoatmega328]"));

    let main_rs = std::fs::read_to_string(proj_path.join("src/main.rs")).unwrap();
    assert!(main_rs.contains("init();"), "Should contain hybrid initialization");
}