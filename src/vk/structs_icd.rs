#![allow(dead_code, bad_style)]

use std::mem;
use libc::*;
use vk::enums::*;

#[repr(C)]
pub struct IcdSurfaceBase {
    pub platform: IcdWsiPlatform
}

pub type IcdWsiPlatform = u32;
pub const ICD_WSI_PLATFORM_MIR: u32 = 0;
pub const ICD_WSI_PLATFORM_WAYLAND: u32 = 1;
pub const ICD_WSI_PLATFORM_WIN32: u32 = 2;
pub const ICD_WSI_PLATFORM_XCB: u32 = 3;
pub const ICD_WSI_PLATFORM_XLIB: u32 = 4;

pub const ICD_LOADER_MAGIC: usize = 0x01CDC0DE;
