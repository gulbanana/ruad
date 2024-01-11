#![cfg_attr(not(test), windows_subsystem = "windows")]

mod config;
mod payload;
pub mod service;
pub mod win32;

use std::{env, io::Write};
use anyhow::{Result, Ok};
use config::Options;

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        if let Err(e) = main_gui() {
            win32::message_box_ok(service::properties::SERVICE_DISPLAY_NAME, e.to_string())?;
        }
        Ok(())
    } else {
        win32::attach_console();
        let result = main_cli(args);
        std::io::stdout().lock().flush().unwrap();
        std::io::stderr().lock().flush().unwrap();
        result
    }
}

/// prompt-driven launch modes
fn main_gui() -> Result<()> {
    let exe = env::current_exe()?;

    if service::is_installed()? {
        if win32::message_box_yesno(service::properties::SERVICE_DISPLAY_NAME, "Do you want to uninstall the service?")? {
            win32::message_box_ok(service::properties::SERVICE_DISPLAY_NAME, service::uninstall()?)?;
        }
    } else {
        if win32::message_box_yesno(service::properties::SERVICE_DISPLAY_NAME, "Do you want to install the service?")? {
            win32::message_box_ok(service::properties::SERVICE_DISPLAY_NAME, service::install(exe)?)?;
        } 
    }

    Ok(())
}

/// argument-driven launch modes
fn main_cli(args: Vec<String>) -> Result<()> {
    let exe = env::current_exe()?;

    match args.len() {
        2 => match args[1].as_str() {
            "/install" => {
                println!("{}", service::install(exe)?);
                Ok(())
            }
            "/uninstall" => {
                println!("{}", service::uninstall()?);
                Ok(())
            }
            "/runOnce" => {
                let options = Options::ensure()?;
                simple_logging::log_to_stderr(options.log_level);
                payload::execute(&options)
            },
            "/runService" => service::run(exe), // passed by SCM           
            _ => print_usage(),
        },
        _ => print_usage(),
    }
}

fn print_usage() -> Result<()> {
    println!(r#"Usage: 
    ruad.exe /install
    ruad.exe /uninstall 
    ruad.exe /runOnce
"#);
    Ok(())
}
