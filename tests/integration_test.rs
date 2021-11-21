use std::{path::PathBuf, process::Command};

use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};

#[test]
fn test1() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/test1.lox");
    let mut cmd = Command::cargo_bin("jlox")?;
    cmd.arg(path);
    cmd.assert().success().stdout(
        r#"one
true
3
"#,
    );

    Ok(())
}

#[test]
fn test2() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/test2.lox");
    let mut cmd = Command::cargo_bin("jlox")?;
    cmd.arg(path);
    cmd.assert().success().stdout(
        r#"inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
"#,
    );

    Ok(())
}
