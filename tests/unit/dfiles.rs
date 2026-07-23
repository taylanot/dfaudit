use std::path::PathBuf;
use dfaudit::explore::dfiles::find_files;


#[test]
fn ignores_wrong_names() {
    let fixture_path = PathBuf::from("tests/fixtures/wrong_names");

    let files = find_files(&fixture_path);

    assert!(files.is_empty(), "similarly-named files should not match");
}

#[test]
fn no_matches() {
    let fixture_path = PathBuf::from("tests/fixtures/c");

    let files = find_files(&fixture_path);

    assert!(files.is_empty());
}

#[test]
fn both_types() {
    let fixture_path = PathBuf::from("tests/fixtures");

    let files = find_files(&fixture_path);

    let mut names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    names.sort();

    assert_eq!(names, vec!["Containerfile", "Dockerfile"]);
}

#[test]
fn dummyfile() {
    let fixture_path = PathBuf::from("tests/fixtures");

    let files = find_files(&fixture_path);

    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(
        !names.contains(&"Dummyfile".to_string()),
        "Dummyfile should never be matched"
    );
    assert_eq!(files.len(), 2, "expected only Dockerfile and Containerfile");
}
