use assert_cmd::prelude::*;
use serde_json;
use std::process::Command;
use std::str;

#[test]
fn echo_hello() -> Result<(), Box<dyn std::error::Error>> {
    let output = 
        Command::cargo_bin("ir")?
        .arg("tests/integration/echo.json")
        .output()?;

    let mut lines = str::from_utf8(&output.stdout)?.lines(); 

    // First line is output from program.
    assert_eq!(lines.next().unwrap(), "Hello, world.");

    // Second line is JSON result.
    let jso: serde_json::Value = serde_json::from_str(lines.next().unwrap())?;
    let proc = &jso["procs"][0];
    assert_eq!(proc["status"], 0);
    let utime = &proc["rusage"]["ru_utime"];
    let utime =
        utime["tv_sec"].as_f64().unwrap()
        + 1e-6 * utime["tv_usec"].as_f64().unwrap();
    assert!(utime > 0.);

    Ok(())
}

