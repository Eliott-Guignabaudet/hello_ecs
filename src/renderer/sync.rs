use std::error::Error;
use std::sync::Arc;
use ash::{vk, Device};

pub struct FrameSync {
    pub image_available_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
    
    device: Arc<Device>
}


impl FrameSync {

    pub fn new(device: Arc<Device>) -> Result<Self, Box<dyn Error>> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let image_available_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }?;

        let fence_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight_fence = unsafe { device.create_fence(&fence_info, None) }?;


        Ok( Self { image_available_semaphore, in_flight_fence, device } )
    }
    

    pub fn wait_for_fence(
        &self,
    ) -> Result<(), Box<dyn Error>> {
        unsafe { self.device.wait_for_fences(&[self.in_flight_fence], true, u64::MAX)?; }
        Ok(())

    }

    pub fn reset_fence (
        &self,
    ) -> Result<(), Box<dyn Error>>  {
        unsafe { self.device.reset_fences(&[self.in_flight_fence])?; }
        Ok(())
    }
}

impl Drop for FrameSync {
    fn drop(&mut self) {
        unsafe { self.device.destroy_fence(self.in_flight_fence, None) }
        unsafe { self.device.destroy_semaphore(self.image_available_semaphore, None) }
    }
}