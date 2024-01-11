mod main;
pub mod properties;

use std::{ffi::OsString, thread::sleep, time::{Duration, Instant}, path::PathBuf};
use anyhow::Result;
use windows::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;
use windows_service::{define_windows_service, service_dispatcher, service::*, service_manager::*};
use properties::*;
use crate::config::Options;
use self::main::service_main;

#[derive(Debug)]
enum Event {
    ExecutePayload,
    StopService
}

pub fn is_installed() -> Result<bool> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::ENUMERATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;    
    let service_access = ServiceAccess::QUERY_CONFIG;

    Ok(service_manager.open_service(SERVICE_NAME, service_access).is_ok())
}

pub fn install(service_binary_path: PathBuf) -> Result<&'static str> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec!["/runService".into()],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };

    let service_access = ServiceAccess::CHANGE_CONFIG | ServiceAccess::START;
    let service = service_manager.create_service(&service_info, service_access)?;
    service.set_description(SERVICE_DESCRIPTION)?;

    service.start::<OsString>(&vec![])?;

    Ok("Service installed and started.")
}

pub fn uninstall() -> Result<&'static str> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // the service will be marked for deletion immediately, but not deleted until it is stopped
    // and open handles are closed; this includes the handle we are using right now!
    service.delete()?;
    if service.query_status()?.current_state != ServiceState::Stopped {
        service.stop()?;
    }
    drop(service);

    // poll for deletion, which may or may not happen quickly
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    while start.elapsed() < timeout {
        if let Err(windows_service::Error::Winapi(e)) =
            service_manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        {
            if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST.0 as i32) {
                return Ok("Service uninstalled.");
            }
        }
        sleep(Duration::from_secs(1));
    }

    Ok("Service marked for uninstallation.")
}

pub fn run(service_binary_path: PathBuf) -> Result<()> {
    let log_file = service_binary_path.with_file_name("log.txt");
    let options = Options::ensure()?;
    simple_logging::log_to_file(log_file, options.log_level)?;
    service_dispatcher::start(SERVICE_NAME, extern_main)?;
    Ok(())
}

// generate boilerplate
define_windows_service!(extern_main, service_main);