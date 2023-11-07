use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::predicate;

/// # ID 查找 Console 输出
///
/// 在命令行中运行
///
/// ```shell
/// maimai-search id 666
/// ```
#[test]
fn id_console() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("id").arg("666");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("デスパレイト"));
    Ok(())
}

/// # Name 查找 Console 输出
///
/// 在命令行中运行
///
/// ```shell
/// maimai-search "消失"
/// ```
#[test]
fn name_console() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("初音 消失");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("初音ミクの消失"));
    Ok(())
}
