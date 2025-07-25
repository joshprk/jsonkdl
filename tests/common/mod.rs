use std::{
    fmt::Write,
    io::Read,
    path::Path,
    process::{Command, Stdio},
};

use miette::{Context, miette};

fn run(command: &mut Command) -> miette::Result<()> {
    let mut child = command
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run binary");

    let status = child.wait().expect("failed to wait for binary");

    let mut buf = String::new();

    child
        .stdout
        .unwrap()
        .read_to_string(&mut buf)
        .expect("failed to read stdout");
    if !buf.ends_with('\n') {
        writeln!(buf).unwrap();
    }

    child
        .stderr
        .unwrap()
        .read_to_string(&mut buf)
        .expect("failed to read stderr");

    let buf = buf.trim();
    if !buf.is_empty() {
        println!("{buf}");
    }

    if status.success() {
        Ok(())
    } else {
        Err(miette!(status))
    }
}

pub fn run_jsonkdl_v1(input: &Path, output: &Path) -> miette::Result<()> {
    run(Command::new(env!("CARGO_BIN_EXE_jsonkdl"))
        .arg("-f")
        .arg("-1")
        .arg(input)
        .arg(output))
    .context(format!("jsonkdl failed on input: {}", input.display()))
}

pub fn run_jsonkdl_v2(input: &Path, output: &Path) -> miette::Result<()> {
    run(Command::new(env!("CARGO_BIN_EXE_jsonkdl"))
        .arg("-f")
        .arg("-2")
        .arg(input)
        .arg(output))
    .context(format!("jsonkdl failed on input: {}", input.display()))
}
