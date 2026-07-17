use std::error::Error;
use ash::{vk, Device};

pub struct FrameSync {
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}


impl FrameSync {

    pub fn new(device: &Device) -> Result<Self, Box<dyn Error>> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let image_available_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }?;
        let render_finished_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }?;

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight_fence = unsafe { device.create_fence(&fence_info, None) }?;


        Ok( Self { image_available_semaphore,  render_finished_semaphore, in_flight_fence } )
    }
    

    pub fn wait_for_fence(
        &self,
        device: &Device,
    ) -> Result<(), Box<dyn Error>> {
        unsafe { device.wait_for_fences(&[self.in_flight_fence], true, u64::MAX)?; }
        Ok(())

    }

    pub fn reset_fence (
        &self,
        device: &Device,
    ) -> Result<(), Box<dyn Error>>  {
        unsafe { device.reset_fences(&[self.in_flight_fence])?; }
        Ok(())
    }
}
