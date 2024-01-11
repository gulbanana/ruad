extern crate native_timer;

use std::{sync::mpsc, time::Duration, ffi::OsString, env::current_exe, panic::catch_unwind};
use anyhow::Result;
use native_timer::*;
use windows_service::{service::*, service_control_handler::{self, ServiceControlHandlerResult}};
use crate::{config::Options, payload};
use super::{properties, Event};

// called on background thread with service parameters parsed by macro
pub fn service_main(_arguments: Vec<OsString>) {
    if let Err(p) = catch_unwind(|| {
        if let Err(e) = event_loop() {
            log::error!("service::main::event_loop | {:#}", e);
        }
    }) {
        let message = match p.downcast::<String>() {
            Ok(v) => *v,
            Err(p) => match p.downcast::<&str>() {
                Ok(v) => v.to_string(),
                _ => "Unknown error".to_owned()
            }
        };
        log::error!("service::main::event_loop | Panic: {:#}", message);
    }
    log::info!("service::main::event_loop | Service stopped");
}

fn event_loop() -> Result<()> {
    let options = Options::ensure()?;
    let mut level = options.log_level;
    let mut interval = options.exec_at_interval;

    // create a channel for signalling the main thread    
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    // register message loop
    let mut control_handler = ControlHandler { event_tx: event_tx.clone(), current_timer: None };
    let status_handle = service_control_handler::register(
        properties::SERVICE_NAME, 
        move |control_event| control_handler.accept(control_event))?;

    // must report status to the system quickly or be killed
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SESSION_CHANGE,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    log::info!("service::main::event_loop | Service started");

    // execute payload periodically, with a single immediate execution at startup
    let poll_tx = event_tx.clone();
    if options.exec_at_startup {
        poll_tx.send(Event::ExecutePayload)?;
    }
    let _timer = schedule_interval(interval, Some(CallbackHint::QuickFunction), move || {
        poll_tx.send(Event::ExecutePayload).unwrap();
    })?;

    // poll for events
    loop {
        let event = event_rx.recv();
        log::trace!("service::main::event_loop | {:?}", event);
        match event {
            Ok(Event::StopService) | Err(mpsc::RecvError) => break,

            Ok(Event::ExecutePayload) => {
                let options = Options::ensure()?;

                if let Err(e) = payload::execute(&options) {
                    log::warn!("payload::execute | {:#}", e);
                }

                if interval != options.exec_at_interval {
                    interval = options.exec_at_interval;
                    _timer.change_period(interval, interval)?;
                }

                if level != options.log_level {
                    level = options.log_level;
                    let log_file = current_exe()?.with_file_name("log.txt");
                    simple_logging::log_to_file(log_file, level)?;
                }
            }
        };
    }

    // politely report impending exit
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

struct ControlHandler {
    event_tx: mpsc::Sender<Event>,
    current_timer: Option<Timer<'static>>
}

impl ControlHandler {
    fn accept(&mut self, control_event: ServiceControl) -> ServiceControlHandlerResult {
        log::trace!("service::main::ControlHandler::accept | {:?}", control_event);
        match self.try_handle(control_event) {
            Ok(handled) => if handled {
                ServiceControlHandlerResult::NoError
            } else {
                ServiceControlHandlerResult::NotImplemented
            },
            Err(e) => {
                log::error!("service::main::ControlHandler::try_handle | {:#}", e);
                ServiceControlHandlerResult::NoError
            }
        }
    }

    fn try_handle(&mut self, control_event: ServiceControl) -> Result<bool> {
        match control_event {
            ServiceControl::Interrogate => Ok(true), // required for all services

            ServiceControl::SessionChange(wts_info) => {
                let options = Options::ensure()?;
                if let (SessionChangeReason::SessionLogon, Some(after)) = (wts_info.reason, options.exec_after_logon) {
                    let schedule_tx = self.event_tx.clone();
                    let timer = schedule_oneshot(after, Some(CallbackHint::QuickFunction), move || {
                        schedule_tx.send(Event::ExecutePayload).unwrap();
                    })?;
                    self.current_timer = Some(timer)
                }
                Ok(true)
            },

            ServiceControl::Stop => {
                self.event_tx.send(Event::StopService)?;
                Ok(true)
            }

            _ => Ok(false)
        }
    }
}
