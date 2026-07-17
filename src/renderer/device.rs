use std::collections::HashSet;
use std::error::Error;
use std::ffi::CStr;
use ash::khr::{surface, swapchain};
use ash::{vk, Device, Entry, Instance};
use thiserror::Error;
use log::{info, warn};
use crate::renderer::command_pool::CommandPool;
use crate::renderer::constants::VALIDATION_LAYER;

const DEVICE_EXTENSIONS: &CStr  = c"VK_KHR_swapchain";

#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn get(
        instance: &Instance,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        surface_loader: &surface::Instance,
    ) -> Result<Self, Box<dyn Error>> {
        unsafe {
            Ok(Self {
                capabilities: surface_loader
                    .get_physical_device_surface_capabilities(
                        physical_device, surface
                    )?,
                formats: surface_loader
                    .get_physical_device_surface_formats(
                        physical_device, surface
                    )?,
                present_modes: surface_loader
                    .get_physical_device_surface_present_modes(
                        physical_device, surface)?,
            })
        }
    }
}

#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);

#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
    pub transfer: u32,
}

impl QueueFamilyIndices {
    pub fn get(
        instance: &ash::Instance,
        surface: vk::SurfaceKHR,
        surface_loader: &surface::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>>{
        let properties = unsafe {
            instance
                .get_physical_device_queue_family_properties(physical_device)
        };

        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        let mut present = None;
        for (index, _) in properties.iter().enumerate() {
            unsafe {
                if surface_loader.get_physical_device_surface_support(
                    physical_device,
                    index as u32,
                    surface,
                )? {
                    present = Some(index as u32);
                    break;
                }
            }
        }

        let transfer = properties
            .iter()
            .position(|p|
                p.queue_flags.contains(vk::QueueFlags::TRANSFER)
                    && !p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);



        if let (Some(graphics), Some(present), Some(transfer)) = (graphics, present, transfer) {
            Ok(Self { graphics, present, transfer })
        } else {
            Err(Box::new(SuitabilityError("Missing required queue families.")))
        }
    }
}


pub struct RenderDevice{
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    
    pub queue_family_indices: QueueFamilyIndices,
    pub swapchain_support: SwapchainSupport,

    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    
    pub transfer_command_pool: CommandPool,
    pub graphics_command_pool: CommandPool,

    pub msaa_samples: vk::SampleCountFlags,
    
    
}

impl RenderDevice {
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        surface: vk::SurfaceKHR,
        surface_loader: &surface::Instance,
    ) -> Result<Self, Box<dyn Error>> {

        let (physical_device, msaa_samples) =
            Self::pick_physical_device(instance, surface, surface_loader)?;

        let (device, queue_family_indices) = Self::create_logical_device(
            entry,
            instance,
            surface,
            physical_device,
            surface_loader)?;

        let (graphics_queue, present_queue, transfer_queue) = Self::get_device_queues(&device, queue_family_indices)?;
        let swapchain_support = SwapchainSupport::get(instance, surface, physical_device, surface_loader)?;
        
        let graphics_command_pool = CommandPool::new(
            &device,
            queue_family_indices.graphics,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?;
        
        let transfer_command_pool = CommandPool::new(
            &device,
            queue_family_indices.transfer,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?;


        Ok(Self {
            physical_device,
            device,
            graphics_queue,
            present_queue,
            transfer_queue,
            transfer_command_pool,
            graphics_command_pool,
            msaa_samples,
            queue_family_indices,
            swapchain_support
        })
    }

    fn pick_physical_device(
        instance: &Instance,
        surface: vk::SurfaceKHR,
        surface_loader: &surface::Instance,
    ) -> Result<(vk::PhysicalDevice, vk::SampleCountFlags), Box<dyn Error>>{
        unsafe {
            for physical_device in instance.enumerate_physical_devices()? {
                let properties = instance.get_physical_device_properties(physical_device);

                if let Err(error) = Self::check_physical_device(instance, physical_device, surface, surface_loader) {
                    warn!("Skipping physical device (`{:?}`): {}", properties.device_name, error);
                } else {
                    info!("Selected physical device (`{:?}`)", properties.device_name);

                    let msaa_samples = Self::get_max_msaa_samples(instance, physical_device);
                    return Ok((physical_device, msaa_samples))
                }
            }
        }

        Err(Box::new( SuitabilityError("Failed to find suitable physical device")))

    }

    fn check_physical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &surface::Instance,
    ) -> Result<(), Box<dyn Error>> {
        QueueFamilyIndices::get(instance, surface, &surface_loader, physical_device)?;
        Self::check_physical_device_extensions(instance, physical_device)?;

        let support = SwapchainSupport::get(instance, surface, physical_device, surface_loader)?;
        if support.formats.is_empty() || support.present_modes.is_empty() {
            return Err(Box::new(SuitabilityError("Insufficient swapchain support.")));
        }

        let features = unsafe { instance.get_physical_device_features(physical_device) };
        if features.sampler_anisotropy != vk::TRUE {
            return Err(Box::new(SuitabilityError("No sampler anisotropy")));
        }

        Ok(())
    }


    fn check_physical_device_extensions(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<(), Box<dyn Error>> {
        let extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
        }?
            .iter()
            .map(|e| e.extension_name)
            .collect::<HashSet<_>>();
        let extensions = &[DEVICE_EXTENSIONS.as_ptr()];
        if extensions.iter().all(|e| extensions.contains(e)) {
            Ok(())
        } else {
            Err(Box::new(SuitabilityError("Missing required device extensions.")))
        }
    }

    fn get_max_msaa_samples(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> vk::SampleCountFlags {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let counts = properties.limits.framebuffer_color_sample_counts
            & properties.limits.framebuffer_depth_sample_counts;
        [
            vk::SampleCountFlags::TYPE_64,
            vk::SampleCountFlags::TYPE_32,
            vk::SampleCountFlags::TYPE_16,
            vk::SampleCountFlags::TYPE_8,
            vk::SampleCountFlags::TYPE_4,
            vk::SampleCountFlags::TYPE_2,
        ]
            .iter()
            .cloned()
            .find(|c| counts.contains(*c))
            .unwrap_or(vk::SampleCountFlags::TYPE_1)
    }


    fn create_logical_device(
        entry: &Entry,
        instance: &Instance,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        surface_loader: &surface::Instance,
    ) -> Result<(Device, QueueFamilyIndices), Box<dyn Error>> {
        let indices = QueueFamilyIndices::get(instance, surface, surface_loader, physical_device)?;

        let mut unique_indices = HashSet::new();
        unique_indices.insert(indices.graphics);
        unique_indices.insert(indices.present);
        unique_indices.insert(indices.transfer);

        let queue_priorities = &[1.0];
        let queue_infos = unique_indices
            .iter()
            .map(|i| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*i)
                    .queue_priorities(queue_priorities)
            })
            .collect::<Vec<_>>();


        let extensions = &[DEVICE_EXTENSIONS.as_ptr()];

        // if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        //     extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
        // }

        let features = vk::PhysicalDeviceFeatures::default()
            .sampler_anisotropy(true)
            .sample_rate_shading(true);

        let info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(extensions)
            .enabled_features(&features);

        let device = unsafe { instance.create_device(physical_device, &info, None) }?;
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        if features.sampler_anisotropy != vk::TRUE {
            return Err(Box::new(SuitabilityError("No sampler anisotropy.")));
        }

        Ok((device, indices))
    }

    fn get_device_queues(
        device: &Device,
        indices: QueueFamilyIndices,
    ) -> Result<(vk::Queue, vk::Queue, vk::Queue), Box<dyn Error>> {
        let graphic_queue = unsafe { device.get_device_queue(indices.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present, 0) };
        let transfer_queue = unsafe { device.get_device_queue(indices.transfer, 0) };

        Ok((graphic_queue, present_queue, transfer_queue))
    }
}