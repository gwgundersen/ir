use assert_cmd::prelude::*;
use serde_json;
use std::process::Command;
use std::str;

#[test]
fn stdout_stderr() -> Result<(), Box<dyn std::error::Error>> {
    let output = 
        Command::cargo_bin("ir")?
        .arg("tests/integration/stdout-stderr.json")
        .output()?;

    let mut lines = str::from_utf8(&output.stdout)?.lines().collect::<Vec<_>>();
    // Last line is JSON result.
    let jso: serde_json::Value = serde_json::from_str(lines.pop().unwrap())?;
    assert_eq!(jso["status"], 42 << 8);  // FIXME

    // FIXME: Where do output files go?

    Ok(())
}

