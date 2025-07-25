use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use kdl::KdlDocument;
use miette::Context;

mod common;

#[test]
fn examples_v1() -> miette::Result<()> {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let output_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("v1");

    fs::create_dir_all(&output_dir).expect("failed to create test output directory");

    for entry in fs::read_dir(examples_dir).expect("failed to read examples directory") {
        let entry = entry.expect("failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let output_file = output_dir.join(format!("{file_name}.kdl"));

        common::run_jsonkdl_v1(&path, &output_file)?;

        let mut kdl_str = String::new();

        File::open(&output_file)
            .expect("failed to open output kdl")
            .read_to_string(&mut kdl_str)
            .expect("failed to read output kdl");

        KdlDocument::parse_v1(&kdl_str).context("output is not valid kdl")?;
    }

    Ok(())
}
