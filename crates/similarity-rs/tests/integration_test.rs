use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_rust_duplicate_detection() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");

    let content = r#"
fn process_items(items: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for item in items {
        if *item > 0 {
            result.push(item * 2);
        }
    }
    result
}

fn handle_items(data: &[i32]) -> Vec<i32> {
    let mut output = Vec::new();
    for d in data {
        if *d > 0 {
            output.push(d * 2);
        }
    }
    output
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.8")
        .assert()
        .success()
        .stdout(predicate::str::contains("process_items"))
        .stdout(predicate::str::contains("handle_items"))
        .stdout(predicate::str::contains("Similarity:"));
}

#[test]
fn test_rust_struct_methods() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_struct.rs");

    let content = r#"
struct DataProcessor {
    data: Vec<i32>,
}

impl DataProcessor {
    fn process(&self) -> Vec<i32> {
        let mut result = Vec::new();
        for item in &self.data {
            result.push(item * 2);
        }
        result
    }
    
    fn transform(&self) -> Vec<i32> {
        let mut output = Vec::new();
        for i in &self.data {
            output.push(i * 2);
        }
        output
    }
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.8")
        .assert()
        .success()
        .stdout(predicate::str::contains("method process"))
        .stdout(predicate::str::contains("method transform"));
}

#[test]
fn test_no_duplicates() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("unique.rs");

    let content = r#"
fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for n in numbers {
        sum += n;
    }
    sum
}

fn find_maximum(values: &[i32]) -> Option<i32> {
    if values.is_empty() {
        return None;
    }
    let mut max = values[0];
    for &v in values.iter().skip(1) {
        if v > max {
            max = v;
        }
    }
    Some(max)
}

fn format_message(name: &str, age: u32) -> String {
    format!("Hello {}, you are {} years old", name, age)
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.85")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"));
}

#[test]
fn test_threshold_filtering() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("threshold_test.rs");

    let content = r#"
fn func1(x: i32) -> i32 {
    let result = x + 1;
    result * 2
}

fn func2(y: i32) -> i32 {
    let temp = y + 1;
    temp * 3  // Different multiplier
}
"#;

    fs::write(&file_path, content).unwrap();

    // With high threshold, should not detect as duplicate
    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.95")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"));
}

#[test]
fn test_min_lines_filtering() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("min_lines_test.rs");

    let content = r#"
fn f1() -> i32 { 1 }
fn f2() -> i32 { 1 }

fn longer_func1() -> i32 {
    let x = 1;
    let y = 2;
    let z = 3;
    x + y + z
}

fn longer_func2() -> i32 {
    let a = 1;
    let b = 2; 
    let c = 3;
    a + b + c
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(&file_path)
        .arg("--min-lines")
        .arg("4")
        .assert()
        .success()
        .stdout(predicate::str::contains("longer_func1"))
        .stdout(predicate::str::contains("longer_func2"))
        .stdout(predicate::str::contains("f1").not());
}
