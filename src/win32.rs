//! Safe wrappers for Windows API functions

use std::ffi::CString;
use anyhow::Result; 
use windows::{
    core::PCSTR,
    Win32::{
        System::Console::{AttachConsole, ATTACH_PARENT_PROCESS},     
        UI::WindowsAndMessaging::{MessageBoxA, MB_OK, MB_YESNO, IDYES}
    }
};

pub fn attach_console() {
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

pub fn message_box_ok(caption: impl Into<Vec<u8>>, msg: impl Into<Vec<u8>>) -> Result<()> {
    let c_caption = CString::new(caption)?;
    let c_msg = CString::new(msg)?;

    unsafe {
        MessageBoxA(
            None, 
            PCSTR::from_raw(c_msg.as_ptr() as *const u8), 
            PCSTR::from_raw(c_caption.as_ptr() as *const u8),
            MB_OK);    
    }

    Ok(())
}

pub fn message_box_yesno(caption: impl Into<Vec<u8>>, msg: impl Into<Vec<u8>>) -> Result<bool> {
    let c_caption = CString::new(caption)?;
    let c_msg = CString::new(msg)?;

    unsafe {
        let button = MessageBoxA(
            None, 
            PCSTR::from_raw(c_msg.as_ptr() as *const u8), 
            PCSTR::from_raw(c_caption.as_ptr() as *const u8),
            MB_YESNO);

        Ok(button == IDYES)
    }
}