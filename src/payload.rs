use std::process::{Command, Stdio};
use anyhow::{Result, Context};

use crate::config::{Options, Payload};

pub fn execute(options: &Options) -> Result<()> {
    let payload = Payload::ensure().context("Locate payload")?;    

    let mut builder = Command::new(&options.reg_cmd);
    builder
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("import")
        .arg(&payload.path);

    let child = builder.spawn().context("Execute reg.exe")?;
    let output = child.wait_with_output()?;    

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        log::info!("payload::execute | reg.exe (stdout):\n{stdout}");
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        log::info!("payload::execute | reg.exe (stderr):\n{stderr}");
    }

    Ok(())
}
