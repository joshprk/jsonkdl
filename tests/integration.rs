use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_examples() {
    let examples_dir = Path::new("examples");
    let output_dir = Path::new("target/test_outputs");

    fs::create_dir_all(output_dir).expect("failed to create test output directory");

    for entry in fs::read_dir(examples_dir).expect("failed to read examples directory") {
        let entry = entry.expect("failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let output_file = output_dir.join(format!("{file_name}.kdl"));

        let status = Command::new(env!("CARGO_BIN_EXE_jsonkdl"))
            .arg(&path)
            .arg(&output_file)
            .arg("-f")
            .status()
            .expect("failed to run binary");

        assert!(status.success(), "jsonkdl failed on input: {:?}", path);
    }
}

