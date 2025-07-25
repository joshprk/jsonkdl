use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::Command;

use kdl::KdlDocument;
use miette::Context;

#[test]
fn examples_v2() -> miette::Result<()> {
    let examples_dir = Path::new("examples");
    let output_dir = Path::new("target/test_outputs/v2");

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
            .arg("-f")
            .arg("-2")
            .arg(&path)
            .arg(&output_file)
            .status()
            .expect("failed to run binary");

        assert!(status.success(), "jsonkdl failed on input: {:?}", path);

        let mut kdl_str = String::new();

        File::open(&output_file)
            .expect("failed to open output kdl")
            .read_to_string(&mut kdl_str)
            .expect("failed to read output kdl");

        KdlDocument::parse_v2(&kdl_str).context("output is not valid kdl")?;
    }

    Ok(())
}
