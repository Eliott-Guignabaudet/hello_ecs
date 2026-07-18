use std::error::Error;
use std::sync::Arc;
use ash::{vk, Device, Instance};
use ash::khr::swapchain;
use crate::renderer::device::{QueueFamilyIndices, SwapchainSupport};
use crate::renderer::utils::create_image_view;

pub struct RenderSwapchain {
    pub swapchain_loader: swapchain::Device,
    pub swapchain : vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    device: Arc<Device>
}

impl RenderSwapchain {
    pub fn new(
        window: &winit::window::Window,
        instance: &Instance,
        device: Arc<Device>,
        surface: vk::SurfaceKHR,
        
        queue_family_indices: QueueFamilyIndices,
        swapchain_support: &SwapchainSupport,
    ) -> Result<Self, Box<dyn Error>> {

        let swapchain_loader = ash::khr::swapchain::Device::new(instance, &device);
        let surface_format = Self::get_swapchain_surface_format(&swapchain_support.formats);
        let present_mode = Self::get_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Self::get_swapchain_extent(window, swapchain_support.capabilities);

        let mut  image_count = swapchain_support.capabilities.min_image_count + 1;
        if swapchain_support.capabilities.max_image_count != 0
            && image_count > swapchain_support.capabilities.max_image_count
        {
            image_count = swapchain_support.capabilities.max_image_count;
        }

        let mut queue_family_unique_indices = vec![];
        let image_sharing_mode = if queue_family_indices.graphics != queue_family_indices.present {
            queue_family_unique_indices.push(queue_family_indices.graphics);
            queue_family_unique_indices.push(queue_family_indices.present);
            vk::SharingMode::CONCURRENT
        } else {
            queue_family_unique_indices.push(queue_family_indices.graphics);
            vk::SharingMode::EXCLUSIVE
        };
        

        let info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_unique_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());
        
        

        let swapchain = unsafe { swapchain_loader.create_swapchain(&info, None) }?;
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;
        let format  = surface_format.format;

        let image_views  = images
            .iter()
            .map(|i|  create_image_view(&device, *i, format, vk::ImageAspectFlags::COLOR, 1))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self{ swapchain_loader, swapchain, format, extent, images, image_views, device})
    }

    fn get_swapchain_surface_format(
        formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| formats[0])
    }

    fn get_swapchain_present_mode(
        present_modes: &Vec<vk::PresentModeKHR>
    ) -> vk::PresentModeKHR {
        present_modes
            .iter()
            .cloned()
            .find(|m| *m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn get_swapchain_extent(
        window: &winit::window::Window,
        capabilities: vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D::default()
                .width(window.inner_size().width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(window.inner_size().height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
        }
    }
    
    pub fn reset(
            &mut self,
            window: &winit::window::Window,
            surface: vk::SurfaceKHR,

            queue_family_indices: QueueFamilyIndices,
            swapchain_support: &SwapchainSupport,
    )-> Result<(), Box<dyn Error>>{
        self.image_views.iter().for_each(|i| {
            unsafe { self.device.destroy_image_view( *i, None) }
        });
        unsafe { self.swapchain_loader.destroy_swapchain(self.swapchain, None) }

        let surface_format = Self::get_swapchain_surface_format(&swapchain_support.formats);
        let present_mode = Self::get_swapchain_present_mode(&swapchain_support.present_modes);
        self.extent = Self::get_swapchain_extent(window, swapchain_support.capabilities);

        let mut  image_count = swapchain_support.capabilities.min_image_count + 1;
        if swapchain_support.capabilities.max_image_count != 0
            && image_count > swapchain_support.capabilities.max_image_count
        {
            image_count = swapchain_support.capabilities.max_image_count;
        }

        let mut queue_family_unique_indices = vec![];
        let image_sharing_mode = if queue_family_indices.graphics != queue_family_indices.present {
            queue_family_unique_indices.push(queue_family_indices.graphics);
            queue_family_unique_indices.push(queue_family_indices.present);
            vk::SharingMode::CONCURRENT
        } else {
            queue_family_unique_indices.push(queue_family_indices.graphics);
            vk::SharingMode::EXCLUSIVE
        };


        let info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(self.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_unique_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());



        self.swapchain = unsafe { self.swapchain_loader.create_swapchain(&info, None) }?;
        self.images = unsafe { self.swapchain_loader.get_swapchain_images(self.swapchain) }?;
        self.format  = surface_format.format;

        self.image_views =  self.images
            .iter()
            .map(|i|  create_image_view(&self.device, *i, self.format, vk::ImageAspectFlags::COLOR, 1))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(())
    }
}

impl Drop for RenderSwapchain {
    fn drop(&mut self) {
        self.image_views.iter().for_each(|i| {
            unsafe { self.device.destroy_image_view( *i, None) }
        });
        
        unsafe { self.swapchain_loader.destroy_swapchain(self.swapchain, None) }
    }
}