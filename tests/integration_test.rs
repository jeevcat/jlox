use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use std::{path::PathBuf, process::Command};

#[test]
fn find_content_in_file() -> Result<(), Box<dyn std::error::Error>> {
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
