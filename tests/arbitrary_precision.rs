use core::f64;
use std::{
    fs::{self, File},
    io::Write,
    num::IntErrorKind,
    path::Path,
};

use kdl::KdlDocument;
use miette::{Context, miette};

mod common;

fn run(name: impl AsRef<Path>, literal: &str) -> miette::Result<()> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("precision");

    fs::create_dir_all(&dir).expect("failed to create test output directory");

    let input = dir.join(&name).with_extension("json");
    let output_v1 = dir.join(&name).with_extension("v1.kdl");
    let output_v2 = dir.join(&name).with_extension("v2.kdl");

    {
        let mut f = File::create(&input).unwrap();
        writeln!(f, r#"[ {{ "name": "-", "arguments": [ {literal} ] }} ]"#).unwrap();
    }

    let (document_v1, document_v2) = {
        let v1 = {
            common::run_jsonkdl_v1(&input, &output_v1)
                .context(format!("failed when converting {literal}"))?;
            let output = fs::read_to_string(output_v1).expect("failed to read output kdl");
            KdlDocument::parse_v1(&output)
                .map_err(miette::Report::new)
                .map_err(|err| miette!("output is not valid kdl v1:\n{err:?}"))
        };

        let v2 = {
            common::run_jsonkdl_v2(&input, &output_v2)
                .context(format!("failed when converting value {literal}"))?;
            let output = fs::read_to_string(output_v2).expect("failed to read output kdl");
            KdlDocument::parse_v2(&output)
                .map_err(miette::Report::new)
                .map_err(|err| miette!("output is not valid kdl v2:\n{err:?}"))
        };

        match (v1, v2) {
            (Ok(v1), Ok(v2)) => (v1, v2),
            (Err(err), Ok(_)) | (Ok(_), Err(err)) => return Err(err),
            (Err(v1), Err(v2)) => {
                return Err(miette!("both outputs are invalid kdl:\n{v1:?}\n{v2:?}\n",));
            }
        }
    };

    for document in [document_v1, document_v2] {
        let [node] = document.nodes() else { panic!() };
        let [entry] = node.entries() else { panic!() };

        let repr = entry
            .format()
            .expect("there should be concrete formatting information after parsing a KDL document")
            .value_repr
            .as_str();

        assert_eq!(literal, repr, "the value has changed during conversion");
    }

    Ok(())
}

// 2^2^2^2^2^2^2^2^2^2^2
const VERY_LARGE_NUMBER: &str = "179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137216";

fn integer(name: &str, literal: &str, error: IntErrorKind) -> miette::Result<()> {
    assert_eq!(
        literal.parse::<i128>().as_ref().map_err(|err| err.kind()),
        Err(&error),
        "test case should fail with the given error"
    );

    run(name, literal)
}

#[test]
fn positive_overflow() -> miette::Result<()> {
    integer(
        "positive_overflow",
        VERY_LARGE_NUMBER,
        IntErrorKind::PosOverflow,
    )
}

#[test]
fn negative_overflow() -> miette::Result<()> {
    integer(
        "positive_overflow",
        &format!("-{VERY_LARGE_NUMBER}"),
        IntErrorKind::NegOverflow,
    )
}

fn floating_point(name: &str, literal: &str, rounds_to: f64) -> miette::Result<()> {
    assert_eq!(
        literal.parse::<f64>(),
        Ok(rounds_to),
        "test case should round to the declared value"
    );
    run(name, &literal)
}

#[test]
fn rounds_to_positive_infinity() -> miette::Result<()> {
    floating_point(
        "rounds_to_positive_infinity",
        &format!("{VERY_LARGE_NUMBER}.0"),
        f64::INFINITY,
    )
}

#[test]
fn rounds_to_negative_infinity() -> miette::Result<()> {
    floating_point(
        "rounds_to_negative_infinity",
        &format!("-{VERY_LARGE_NUMBER}.0"),
        f64::NEG_INFINITY,
    )
}

#[test]
fn rounds_to_zero() -> miette::Result<()> {
    floating_point(
        "rounds_to_zero",
        "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001",
        0.0,
    )
}

#[test]
fn rounds_to_one() -> miette::Result<()> {
    floating_point("rounds_to_one", "1.000000000000000000001", 1.0)
}

#[test]
fn rounds_to_two() -> miette::Result<()> {
    floating_point(
        "rounds_to_two",
        "1.999999999999999999999999999999999999999999999999999999999999",
        2.0,
    )
}

#[test]
fn rounds_to_positive_infinity_exp() -> miette::Result<()> {
    floating_point(
        "rounds_to_positive_infinity_exp",
        "1e10000000",
        f64::INFINITY,
    )
}

#[test]
fn rounds_to_negative_infinity_exp() -> miette::Result<()> {
    floating_point(
        "rounds_to_negative_infinity_exp",
        "-1e10000000",
        f64::NEG_INFINITY,
    )
}

#[test]
fn rounds_to_zero_exp() -> miette::Result<()> {
    floating_point("rounds_to_zero_exp", "1e-10000000", 0.0)
}
