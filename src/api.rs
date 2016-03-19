#![allow(non_snake_case)]

use {std, vk, env_logger, libc, alloc};

use {PhysicalDevice, Device, Instance, Queue, Swapchain, CommandBuffer, Dispatched};

macro_rules! entrypoints {
    (
        $($name:ident ( $( $pname:ident : $pty:ty ),* ) -> $rty:ty => $code:block)*
    ) => (
        #[allow(unused_variables)]
        $(
            pub fn $name($($pname : $pty),*) -> $rty {
                debug!("Called {}", stringify!($name));
                $code
            }
        )*
        fn symbol_to_function(symbol: &[u8]) -> Option<vk::PFN_vkVoidFunction> {
            $(
                if concat!("vk", stringify!($name)).as_bytes() == symbol {
                    return Some(unsafe { std::mem::transmute($name) });
                }
            )*
            None
        }
    )
}

entrypoints! {
    CreateInstance(create_info: *const vk::InstanceCreateInfo,
                   allocator: *const vk::AllocationCallbacks,
                   vk_instance: *mut vk::Instance) -> vk::Result =>
    {
        use {Dispatched, Instance};

        if !allocator.is_null() {
            warn!("CreateInstance: ignoring request for custom allocator");
        }

        unsafe {
            info!("CreateInstance: requesting {} extensions", (*create_info).enabledExtensionCount);
            let exts = std::slice::from_raw_parts((*create_info).ppEnabledExtensionNames,
                                                  (*create_info).enabledExtensionCount as usize);
            for &ext_p in exts {
                let name = std::ffi::CStr::from_ptr(ext_p);

                info!("CreateInstance: requesting {}", name.to_str().unwrap());
                if name.to_bytes() != b"VK_KHR_surface" {
                    return vk::ERROR_EXTENSION_NOT_PRESENT;
                }
            }
        }

        let instance = Box::new(Dispatched::new(Instance::new()));
        unsafe {
            *vk_instance = Box::into_raw(instance) as usize;
        }

        vk::SUCCESS
    }

    DestroyInstance(vk_instance: vk::Instance, allocator: *const vk::AllocationCallbacks) -> () => {
        use {Dispatched, Instance};

        unsafe {
            Box::<Dispatched<Instance>>::from_raw(vk_instance as *mut Dispatched<Instance>);
        }
    }

    EnumerateInstanceExtensionProperties(layer_name: *const libc::c_char, property_count: *mut u32,
                                         properties: *mut vk::ExtensionProperties) -> vk::Result =>
    {
        if !layer_name.is_null() {
            unsafe {
                *property_count = 0;
            }
            return vk::SUCCESS;
        }

        unsafe {
            do_list(
                &[
                    vk::ExtensionProperties {
                        extensionName: padb256(b"VK_KHR_surface"),
                        specVersion: 25
                    }
                ],
                property_count, properties
            )
        }
    }

    GetDeviceProcAddr(device: vk::Device, name: *const libc::c_char) -> vk::PFN_vkVoidFunction => {
        unsafe {
            let name = std::ffi::CStr::from_ptr(name);

            match symbol_to_function(name.to_bytes()) {
                Some(f) => f,
                None => { warn!("GetDeviceProcAddr: Unknown symbol {}", name.to_str().unwrap()); std::mem::transmute(0usize) }
            }
        }
    }

    EnumeratePhysicalDevices(vk_instance: vk::Instance, physical_device_count: *mut u32,
                             physical_devices: *mut vk::PhysicalDevice) -> vk::Result =>
    {
        use {Dispatched, Instance};

        unsafe {
            let instance: &'static Dispatched<Instance> = std::mem::transmute(vk_instance);

            do_list(&instance.physical_devices(), physical_device_count, physical_devices)
        }
    }

    GetPhysicalDeviceFormatProperties(physical_device: vk::PhysicalDevice, format: vk::Format,
                                      format_properties: *mut vk::FormatProperties) -> () =>
    {
    }

    GetPhysicalDeviceImageFormatProperties(physical_device: vk::PhysicalDevice, format: vk::Format,
                                           type_: vk::ImageType, tiling: vk::ImageTiling,
                                           usage: vk::ImageUsageFlags, flags: vk::ImageCreateFlags,
                                           image_format_props: *mut vk::ImageFormatProperties)
    -> vk::Result => {
        vk::ERROR_FORMAT_NOT_SUPPORTED
    }

    GetPhysicalDeviceProperties(physical_device: vk::PhysicalDevice,
                                properties: *mut vk::PhysicalDeviceProperties) -> () =>
    {
        unsafe {
            (*properties).apiVersion = (1u32 << 22) | (0u32 << 12) | (3u32 << 0);
            (*properties).driverVersion = 0x1;
            (*properties).vendorID = 0;
            (*properties).deviceID = 0;
            (*properties).deviceType = vk::PHYSICAL_DEVICE_TYPE_CPU;

            copy_slice(b"SoftVK Renderer\0", &mut (*properties).deviceName);
        }
    }

    GetPhysicalDeviceQueueFamilyProperties(vk_pdev: vk::PhysicalDevice,
                                           prop_count: *mut u32,
                                           props: *mut vk::QueueFamilyProperties) -> () =>
    {
        unsafe {
            let pdev: &'static Dispatched<PhysicalDevice> = std::mem::transmute(vk_pdev);
            do_list(
                &pdev.queue_families(),
                prop_count, props);
        }
    }

    GetPhysicalDeviceMemoryProperties(physical_device: vk::PhysicalDevice,
                                      memory_props: *mut vk::PhysicalDeviceMemoryProperties)
    -> () => {
        unsafe {
            (*memory_props).memoryTypeCount = 0;
            (*memory_props).memoryHeapCount = 0;
        }
    }

    GetPhysicalDeviceFeatures(physical_device: vk::PhysicalDevice,
                              features: *mut vk::PhysicalDeviceFeatures) -> () =>
    {
        // Disable all features
        unsafe {
            *features = std::mem::zeroed();
        }
    }

    GetPhysicalDeviceSparseImageFormatProperties(physical_device: vk::PhysicalDevice,
                                                 format: vk::Format,
                                                 type_: vk::ImageType,
                                                 samples: vk::SampleCountFlagBits,
                                                 usage: vk::ImageUsageFlags,
                                                 tiling: vk::ImageTiling,
                                                 prop_count: *mut u32,
                                                 props: *mut vk::SparseImageFormatProperties)
    -> () => {
        unsafe {
            *prop_count = 0;
        }
    }

    EnumerateDeviceExtensionProperties(physical_device: vk::PhysicalDevice,
                                       layer_name: *const libc::c_char, prop_count: *mut u32,
                                       props: *mut vk::ExtensionProperties) -> vk::Result =>
    {
        if !layer_name.is_null() {
            unsafe {
                *prop_count = 0;
            }
            return vk::SUCCESS;
        }

        unsafe {
            do_list(
                &[
                    vk::ExtensionProperties {
                        extensionName: padb256(b"VK_KHR_swapchain"),
                        specVersion: 67
                    }
                ],
                prop_count, props
            )
        }
    }

    CreateDevice(physical_device: vk::PhysicalDevice, create_info: *const vk::DeviceCreateInfo,
                 allocator: *const vk::AllocationCallbacks, device: *mut vk::Device) -> vk::Result
    => {
        use {Dispatched, PhysicalDevice, Device};

        if !allocator.is_null() {
            warn!("CreateDevice: ignoring request for custom allocator");
        }

        unsafe {
            let pdev: &'static Dispatched<PhysicalDevice> = std::mem::transmute(physical_device);
            let dev: Box<Dispatched<Device>> = pdev.create_device();
            *device = Box::into_raw(dev) as usize;
        }

        vk::SUCCESS
    }

    DestroyDevice(device: vk::Device, allocator: *const vk::AllocationCallbacks) -> () => {
        use Device;

        unsafe { Box::<Dispatched<Device>>::from_raw(device as *mut Dispatched<Device>); }
    }

    // VK_KHR_surface

    GetPhysicalDeviceSurfaceSupportKHR(vk_pdev: vk::PhysicalDevice, queue_family: u32,
                                       vk_surface: vk::SurfaceKHR, supported: *mut vk::Bool32)
    -> vk::Result => {
        use PhysicalDevice;

        unsafe {
            let pdev: &'static Dispatched<PhysicalDevice> = std::mem::transmute(vk_pdev);
            let surf: &'static vk::IcdSurfaceBase = std::mem::transmute(vk_surface);

            info!("GetPhysicalDeviceSurfaceSupportKHR: platform = {}", surf.platform);

            if surf.platform == vk::ICD_WSI_PLATFORM_XCB {
                *supported = vk::TRUE;
            } else {
                *supported = vk::FALSE;
            }
        }

        vk::SUCCESS
    }

    GetPhysicalDeviceSurfaceCapabilitiesKHR(vk_pdev: vk::PhysicalDevice, vk_surface: vk::SurfaceKHR,
                                            caps: *mut vk::SurfaceCapabilitiesKHR)
    -> vk::Result => {
        unsafe {
            (*caps).minImageCount = 1;
            (*caps).maxImageCount = 1;
            (*caps).currentExtent.width = 256;
            (*caps).currentExtent.height = 256;
            (*caps).minImageExtent.width = 256;
            (*caps).minImageExtent.height = 256;
            (*caps).maxImageExtent.width = 256;
            (*caps).maxImageExtent.height = 256;
            (*caps).maxImageArrayLayers = 1;
            (*caps).supportedTransforms = vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR;
            (*caps).currentTransform = vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR;
            (*caps).supportedCompositeAlpha = vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
            (*caps).supportedUsageFlags = vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
        }

        vk::SUCCESS
    }

    GetPhysicalDeviceSurfaceFormatsKHR(vk_pdev: vk::PhysicalDevice, vk_surface: vk::SurfaceKHR,
                                       format_count: *mut u32, formats: *mut vk::SurfaceFormatKHR)
    -> vk::Result => {
        unsafe {
            do_list(&[
                vk::SurfaceFormatKHR {
                    format: vk::FORMAT_B8G8R8A8_UNORM,
                    colorSpace: vk::COLORSPACE_SRGB_NONLINEAR_KHR
                }
            ], format_count, formats)
        }
    }

    GetPhysicalDeviceSurfacePresentModesKHR(vk_pdev: vk::PhysicalDevice, vk_surface: vk::SurfaceKHR,
                                            mode_count: *mut u32, modes: *mut vk::PresentModeKHR)
    -> vk::Result => {
        unsafe {
            do_list(&[], mode_count, modes)
        }
    }

    GetPhysicalDeviceXcbPresentationSupportKHR(vk_pdev: vk::PhysicalDevice, queue_family: u32,
                                               xcb_conn: *mut (), xcb_visual_id: u32)
    -> vk::Bool32 => {
        vk::FALSE
    }

    GetDeviceQueue(vk_dev: vk::Device, queue_family: u32, queue_id: u32, ptr: *mut vk::Queue)
    -> () => {
        unsafe {
            let dev: &'static Dispatched<Device> = std::mem::transmute(vk_dev);
            let queue: Box<Dispatched<Queue>> = dev.create_queue(queue_family, queue_id).unwrap();
            *ptr = Box::into_raw(queue) as usize;
        }
    }

    // VK_KHR_swapchain

    CreateSwapchainKHR(vk_dev: vk::Device, create_info: *const vk::SwapchainCreateInfoKHR,
                       allocator: *const vk::AllocationCallbacks, ptr: *mut vk::SwapchainKHR)
    -> vk::Result => {
        if !allocator.is_null() {
            warn!("CreateSwapchainKHR: ignoring request for custom allocator");
        }

        unsafe {
            let dev: &'static Dispatched<Device> = std::mem::transmute(vk_dev);
            let swapchain: Box<Swapchain> = dev.create_swapchain();
            *ptr = Box::into_raw(swapchain) as u64;
        }

        vk::SUCCESS
    }

    GetSwapchainImagesKHR(vk_dev: vk::Device, swapchain: vk::SwapchainKHR, image_count: *mut u32,
                          image: *mut vk::Image)
    -> vk::Result => {
        unsafe {
            do_list(&[40], image_count, image)
        }
    }

    // Command buffers

    CreateCommandPool(vk_dev: vk::Device, create_info: *const vk::CommandPoolCreateInfo,
                      allocator: *const vk::AllocationCallbacks, ptr: *mut vk::CommandPool)
    -> vk::Result => {
        unsafe {
            *ptr = 41;
        }

        vk::SUCCESS
    }

    AllocateCommandBuffers(vk_dev: vk::Device, info: *const vk::CommandBufferAllocateInfo,
                           command_buffers: *mut vk::CommandBuffer)
    -> vk::Result => {
        unsafe {
            let dev: &'static Dispatched<Device> = std::mem::transmute(vk_dev);

            for i in 0..(*info).commandBufferCount {
                let buffer: Box<Dispatched<CommandBuffer>> = dev.create_command_buffer();
                *command_buffers.offset(i as isize) = Box::into_raw(buffer) as usize;
            }
        }

        vk::SUCCESS
    }

    BeginCommandBuffer(vk_buf: vk::CommandBuffer, info: *const vk::CommandBufferBeginInfo)
    -> vk::Result => {
        vk::SUCCESS
    }

    // Images

    CreateImage(vk_dev: vk::Device, info: *const vk::ImageCreateInfo,
                allocator: *const vk::AllocationCallbacks, ptr: *mut vk::Image)
    -> vk::Result => {
        if !allocator.is_null() {
            warn!("CreateImage: ignoring request for custom allocator");
        }

        unsafe {
            *ptr = 43;
        }

        vk::SUCCESS
    }

    CreateImageView(vk_dev: vk::Device, info: *const vk::ImageViewCreateInfo,
                    allocator: *const vk::AllocationCallbacks, ptr: *mut vk::ImageView)
    -> vk::Result => {
        if !allocator.is_null() {
            warn!("CreateImageView: ignoring request for custom allocator");
        }

        unsafe {
            *ptr = 42;
        }

        vk::SUCCESS
    }

    // Stubs

    QueueSubmit() -> () => { }
    QueueWaitIdle() -> () => { }
    DeviceWaitIdle() -> () => { }
    AllocateMemory() -> () => { }
    FreeMemory() -> () => { }
    MapMemory() -> () => { }
    UnmapMemory() -> () => { }
    FlushMappedMemoryRanges() -> () => { }
    InvalidateMappedMemoryRanges() -> () => { }
    GetDeviceMemoryCommitment() -> () => { }
    GetImageSparseMemoryRequirements() -> () => { }
    GetBufferMemoryRequirements() -> () => { }
    GetImageMemoryRequirements() -> () => { }
    BindBufferMemory() -> () => { }
    BindImageMemory() -> () => { }
    QueueBindSparse() -> () => { }
    CreateFence() -> () => { }
    DestroyFence() -> () => { }
    ResetFences() -> () => { }
    GetFenceStatus() -> () => { }
    WaitForFences() -> () => { }
    CreateSemaphore() -> () => { }
    DestroySemaphore() -> () => { }
    CreateEvent() -> () => { }
    DestroyEvent() -> () => { }
    GetEventStatus() -> () => { }
    SetEvent() -> () => { }
    ResetEvent() -> () => { }
    CreateQueryPool() -> () => { }
    DestroyQueryPool() -> () => { }
    GetQueryPoolResults() -> () => { }
    CreateBuffer() -> () => { }
    DestroyBuffer() -> () => { }
    CreateBufferView() -> () => { }
    DestroyBufferView() -> () => { }
    DestroyImage() -> () => { }
    GetImageSubresourceLayout() -> () => { }
    DestroyImageView() -> () => { }
    CreateShaderModule() -> () => { }
    DestroyShaderModule() -> () => { }
    CreatePipelineCache() -> () => { }
    DestroyPipelineCache() -> () => { }
    GetPipelineCacheData() -> () => { }
    MergePipelineCaches() -> () => { }
    CreateGraphicsPipelines() -> () => { }
    CreateComputePipelines() -> () => { }
    DestroyPipeline() -> () => { }
    CreatePipelineLayout() -> () => { }
    DestroyPipelineLayout() -> () => { }
    CreateSampler() -> () => { }
    DestroySampler() -> () => { }
    CreateDescriptorSetLayout() -> () => { }
    DestroyDescriptorSetLayout() -> () => { }
    CreateDescriptorPool() -> () => { }
    DestroyDescriptorPool() -> () => { }
    ResetDescriptorPool() -> () => { }
    AllocateDescriptorSets() -> () => { }
    FreeDescriptorSets() -> () => { }
    UpdateDescriptorSets() -> () => { }
    CreateFramebuffer() -> () => { }
    DestroyFramebuffer() -> () => { }
    CreateRenderPass() -> () => { }
    DestroyRenderPass() -> () => { }
    GetRenderAreaGranularity() -> () => { }
    DestroyCommandPool() -> () => { }
    ResetCommandPool() -> () => { }
    FreeCommandBuffers() -> () => { }
    EndCommandBuffer() -> () => { }
    ResetCommandBuffer() -> () => { }
    CmdBindPipeline() -> () => { }
    CmdSetViewport() -> () => { }
    CmdSetScissor() -> () => { }
    CmdSetLineWidth() -> () => { }
    CmdSetDepthBias() -> () => { }
    CmdSetBlendConstants() -> () => { }
    CmdSetDepthBounds() -> () => { }
    CmdSetStencilCompareMask() -> () => { }
    CmdSetStencilWriteMask() -> () => { }
    CmdSetStencilReference() -> () => { }
    CmdBindDescriptorSets() -> () => { }
    CmdBindVertexBuffers() -> () => { }
    CmdBindIndexBuffer() -> () => { }
    CmdDraw() -> () => { }
    CmdDrawIndexed() -> () => { }
    CmdDrawIndirect() -> () => { }
    CmdDrawIndexedIndirect() -> () => { }
    CmdDispatch() -> () => { }
    CmdDispatchIndirect() -> () => { }
    CmdCopyBuffer() -> () => { }
    CmdCopyImage() -> () => { }
    CmdBlitImage() -> () => { }
    CmdCopyBufferToImage() -> () => { }
    CmdCopyImageToBuffer() -> () => { }
    CmdUpdateBuffer() -> () => { }
    CmdFillBuffer() -> () => { }
    CmdClearColorImage() -> () => { }
    CmdClearDepthStencilImage() -> () => { }
    CmdClearAttachments() -> () => { }
    CmdResolveImage() -> () => { }
    CmdSetEvent() -> () => { }
    CmdResetEvent() -> () => { }
    CmdWaitEvents() -> () => { }
    CmdPipelineBarrier() -> () => { }
    CmdBeginQuery() -> () => { }
    CmdEndQuery() -> () => { }
    CmdResetQueryPool() -> () => { }
    CmdWriteTimestamp() -> () => { }
    CmdCopyQueryPoolResults() -> () => { }
    CmdPushConstants() -> () => { }
    CmdBeginRenderPass() -> () => { }
    CmdNextSubpass() -> () => { }
    CmdEndRenderPass() -> () => { }
    CmdExecuteCommands() -> () => { }
//     AcquireNextImageKHR() -> () => { }
//     CreateSwapchainKHR() -> () => { }
//     DestroySwapchainKHR() -> () => { }
//     GetSwapchainImagesKHR() -> () => { }
//     QueuePresentKHR() -> () => { }
}

fn padb256(s: &[u8]) -> [i8; 256] {
    let mut r = [0i8; 256];
    for i in 0..s.len() {
        r[i] = s[i] as i8;
    }
    r
}

unsafe fn copy_slice(text: &[u8], target: &mut [i8]) {
    for i in 0..text.len() {
        target[i] = text[i] as i8;
    }
}

unsafe fn do_list<T>(list: &[T], count: *mut u32, ptr: *mut T) -> vk::Result {
    if ptr.is_null() {
        *count = list.len() as u32;
        vk::SUCCESS
    } else {
        let to_copy = std::cmp::min(*count as usize, list.len());
        *count = to_copy as u32;
        std::ptr::copy_nonoverlapping(list.as_ptr(), ptr, to_copy);
        if to_copy == list.len() {
            vk::SUCCESS
        } else {
            vk::INCOMPLETE
        }
    }
}

#[no_mangle]
pub extern fn unknown_method() {
    error!("Unknown method called");
}

#[no_mangle]
pub extern fn vk_icdGetInstanceProcAddr(inst: vk::Instance, name: *const libc::c_char) -> vk::PFN_vkVoidFunction {
    env_logger::init();
//     println!("Called vk_icdGetInstanceProcAddress");
    unsafe {
        let name = std::ffi::CStr::from_ptr(name);

        match symbol_to_function(name.to_bytes()) {
            Some(f) => f,
            None => { warn!("Unknown symbol {}", name.to_str().unwrap()); std::mem::transmute(0usize) }
        }
    }
}
