use assert_cmd::Command;
use tempfile::tempdir;
use predicates::prelude::*;

#[test]
fn test_cli_boards_json_output() {
    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.arg("boards")
       .arg("--json-output");

    // output should succeed and contain JSON format
    cmd.assert()
       .success()
       .stdout(predicates::str::contains("\"id\": \"uno\""))
       .stdout(predicates::str::contains("\"mcu\": \"ATMEGA328P\""));
}

#[test]
fn test_cli_project_init_pure_mode() {
    // create a temporary directory that deletes itself after the test
    let temp = tempdir().unwrap();
    let project_name = "test_pure_blink";

    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.current_dir(&temp)
       .arg("project")
       .arg("init")
       .arg(project_name)
       .arg("--board")
       .arg("uno");

    // init succeeds
    cmd.assert().success();

    let proj_path = temp.path().join(project_name);
    
    // check if the file structure was created
    assert!(proj_path.exists());
    assert!(proj_path.join("Cargo.toml").exists());
    assert!(proj_path.join("Prustio.toml").exists());
    assert!(proj_path.join(".cargo/config.toml").exists());
    assert!(proj_path.join("rust-toolchain.toml").exists());
    assert!(proj_path.join("src/main.rs").exists());

    // verify Prustio.toml contents
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
       .arg("--hybrid"); 

    cmd.assert().success();

    let proj_path = temp.path().join(project_name);
    
    let prustio_toml = std::fs::read_to_string(proj_path.join("Prustio.toml")).unwrap();
    assert!(prustio_toml.contains("hybrid_mode = true"));
    assert!(prustio_toml.contains("[env.nanoatmega328]"));

    let main_rs = std::fs::read_to_string(proj_path.join("src/main.rs")).unwrap();
    assert!(main_rs.contains("init();"), "Should contain hybrid initialization");
}

#[test]
fn test_cli_boards_filtered_output() {
    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.arg("boards")
       .arg("nano") 
       .arg("--json-output");

    // should contain nano boards, but NOT contain uno
    cmd.assert()
       .success()
       .stdout(predicates::str::contains("\"id\": \"nanoatmega328\""))
       .stdout(predicates::str::contains("\"id\": \"uno\"").not());
}

#[test]
fn test_cli_project_init_invalid_board() {
    let temp = tempdir().unwrap();
    let project_name = "test_invalid_board";

    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.current_dir(&temp)
       .arg("project")
       .arg("init")
       .arg(project_name)
       .arg("--board")
       .arg("this_board_does_not_exist_123");

    // command should fail because the board is unknown
    cmd.assert()
       .failure()
       .stderr(predicates::str::contains("Unsupported")); 
}

#[test]
fn test_cli_clean_project() {
    let temp = tempdir().unwrap();
    let project_name = "test_clean_proj";

    // unitialize a pure mode project
    let mut init_cmd = Command::cargo_bin("prustio").unwrap();
    init_cmd.current_dir(&temp)
            .arg("project")
            .arg("init")
            .arg(project_name)
            .arg("--board")
            .arg("uno");
    init_cmd.assert().success();

    let proj_path = temp.path().join(project_name);

    // Mock some build artifact that `prustio clean` remove
    let target_dir = proj_path.join("target");
    let prio_dir = proj_path.join(".prio");
    std::fs::create_dir_all(&target_dir).unwrap();
    std::fs::create_dir_all(&prio_dir).unwrap();

    assert!(target_dir.exists());
    assert!(prio_dir.exists());

    // run the clean command
    let mut clean_cmd = Command::cargo_bin("prustio").unwrap();
    clean_cmd.current_dir(&proj_path)
             .arg("clean");
    
    clean_cmd.assert().success();
}

#[test]
fn test_cli_run_outside_project() {
    let temp = tempdir().unwrap();

    // attempting to run project-specific commands
    let mut cmd = Command::cargo_bin("prustio").unwrap();
    cmd.current_dir(&temp)
       .arg("run");

    // it should fail
    cmd.assert()
       .failure();
}