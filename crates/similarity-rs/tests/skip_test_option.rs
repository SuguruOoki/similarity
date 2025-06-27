use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_skip_test_option() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("lib.rs");

    // Create a file with both test and non-test functions
    fs::write(
        &file1,
        r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

fn calculate_product(a: i32, b: i32) -> i32 {
    a * b
}

#[test]
fn test_calculate_sum() {
    assert_eq!(calculate_sum(2, 3), 5);
}

#[test]
fn test_calculate_product() {
    assert_eq!(calculate_product(2, 3), 6);
}

fn test_helper_function() -> bool {
    true
}

fn another_test_helper() -> bool {
    true
}
"#,
    )
    .unwrap();

    // Run without --skip-test (should find duplicates in test functions)
    let mut cmd = Command::cargo_bin("similarity-rs").unwrap();
    cmd.arg(dir.path());

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should find duplicates among test functions
    assert!(
        stdout.contains("test_calculate_sum")
            || stdout.contains("test_calculate_product")
            || stdout.contains("test_helper_function")
    );

    // Run with --skip-test (should not find test functions)
    let mut cmd = Command::cargo_bin("similarity-rs").unwrap();
    cmd.arg(dir.path()).arg("--skip-test");

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should not find any test functions
    assert!(!stdout.contains("test_calculate_sum"));
    assert!(!stdout.contains("test_calculate_product"));
    assert!(!stdout.contains("test_helper_function"));

    // Should still find non-test functions
    assert!(
        stdout.contains("calculate_sum")
            || stdout.contains("calculate_product")
            || stdout.contains("another_test_helper")
    );
}

#[test]
fn test_skip_test_with_test_attribute() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("tests.rs");

    // Create a file with functions that have #[test] attribute
    fs::write(
        &file1,
        r#"
#[test]
fn should_be_skipped() {
    let x = 1;
    let y = 2;
    assert_eq!(x + y, 3);
}

#[test]
fn also_should_be_skipped() {
    let x = 1;
    let y = 2;
    assert_eq!(x + y, 3);
}

fn normal_function() {
    let x = 1;
    let y = 2;
    println!("{}", x + y);
}

fn another_normal_function() {
    let x = 1;
    let y = 2;
    println!("{}", x + y);
}
"#,
    )
    .unwrap();

    // Run with --skip-test
    let mut cmd = Command::cargo_bin("similarity-rs").unwrap();
    cmd.arg(dir.path()).arg("--skip-test");

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should not find functions with #[test] attribute
    assert!(!stdout.contains("should_be_skipped"));
    assert!(!stdout.contains("also_should_be_skipped"));

    // Should find normal functions
    assert!(stdout.contains("normal_function") && stdout.contains("another_normal_function"));
}
