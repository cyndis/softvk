#![feature(alloc, heap_api, associated_type_defaults)]

extern crate libc;
extern crate alloc;

#[macro_use]
extern crate log;
extern crate env_logger;

mod vk {
    mod structs;
    mod enums;
    mod structs_icd;

    pub use self::structs::*;
    pub use self::enums::*;
    pub use self::structs_icd::*;
}

pub mod api;

pub struct Dispatched<T> {
    magic: usize,
    data: T
}

impl<T> Dispatched<T> {
    pub fn new(data: T) -> Dispatched<T> {
        Dispatched {
            magic: vk::ICD_LOADER_MAGIC,
            data: data
        }
    }

    pub fn handle(&self) -> usize {
        self as *const Self as usize
    }
}

impl<T> std::ops::Deref for Dispatched<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> std::ops::DerefMut for Dispatched<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

pub struct Queue;

pub struct Device;

pub struct Swapchain;

pub struct CommandBuffer;

impl Device {
    pub fn create_queue(&self, family: u32, id: u32) -> Option<Box<Dispatched<Queue>>> {
        Some(Box::new(Dispatched::new(Queue)))
    }

    pub fn create_swapchain(&self) -> Box<Swapchain> {
        Box::new(Swapchain)
    }

    pub fn create_command_buffer(&self) -> Box<Dispatched<CommandBuffer>> {
        Box::new(Dispatched::new(CommandBuffer))
    }
}

pub struct PhysicalDevice;

impl PhysicalDevice {
    pub fn create_device(&self) -> Box<Dispatched<Device>> {
        Box::new(Dispatched::new(Device))
    }

    pub fn queue_families(&self) -> Vec<vk::QueueFamilyProperties> {
        vec![
            vk::QueueFamilyProperties {
                queueFlags: vk::QUEUE_GRAPHICS_BIT,
                queueCount: 1,
                timestampValidBits: 0,
                minImageTransferGranularity: vk::Extent3D {
                    width: 0,
                    height: 0,
                    depth: 0
                }
            }
        ]
    }
}

pub struct Instance {
    physical_device: Dispatched<PhysicalDevice>
}

impl Instance {
    pub fn new() -> Instance {
        Instance {
            physical_device: Dispatched::new(PhysicalDevice)
        }
    }

    pub fn physical_devices(&self) -> Vec<vk::PhysicalDevice> {
        vec![self.physical_device.handle()]
    }
}
