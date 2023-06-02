#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use vulkano::descriptor_set::allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo};
use vulkano::device::{DeviceExtensions, QueueFlags};
use vulkano::image::ImageUsage;
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{GenericMemoryAllocator, StandardMemoryAllocator};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError};
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use log::*;

use std::sync::Arc;

pub struct Context {
    pub event_loop: EventLoop<()>,
    pub window: Window,
    pub instance: Arc<Instance>,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub surface: Arc<Surface>,
    pub swapchain: Arc<Swapchain>,
    pub swapchain_images: Vec<Arc<SwapchainImage>>,
    pub queue: Arc<Queue>,
    pub allocator: Arc<StandardMemoryAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl Context {
    pub fn new() -> Context {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .build(&event_loop)
            .expect("CONTEXT | failed to create window");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let library = VulkanLibrary::new().expect("CONTEXT | no local Vulkan library");
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: vulkano_win::required_extensions(&library),
                ..Default::default()
            },
        )
        .expect("CONTEXT | failed to create instance");
        let physical_device = instance
            .enumerate_physical_devices()
            .expect("CONTEXT | failed to enumerate vulkan devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .nth(0)
            .expect("CONTEXT | no suitable physical device found");
        info!(
            "CONTEXT | physical device name: {}",
            physical_device.properties().device_name
        );
        for family in physical_device.queue_family_properties() {
            info!(
                "CONTEXT | queue family with {:?} queue(s) found",
                family.queue_count
            );
        }
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .expect("CONTEXT | couldn't find a graphical queue family")
            as u32;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: DeviceExtensions {
                    khr_swapchain: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("CONTEXT | failed to create device");
        let queue = queues.next().unwrap();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .expect("CONTEXT | failed to create vulkan surface");
        let device_capabilities = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("CONTEXT | failed to get surface capabilities");
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: device_capabilities.min_image_count + 1,
                image_format: Some(
                    physical_device
                        .surface_formats(&surface, Default::default())
                        .unwrap()[0]
                        .0,
                ),
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: device_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .expect("CONTEXT | failed to create swapchain");
        let allocator = StandardMemoryAllocator::new_default(device.clone());
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());

        Context {
            event_loop,
            window,
            instance,
            physical_device,
            device,
            surface,
            swapchain,
            swapchain_images: images,
            queue,
            allocator: Arc::new(allocator),
            descriptor_set_allocator: Arc::new(descriptor_set_allocator),
        }
    }
}
