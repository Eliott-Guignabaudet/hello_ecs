mod constants;
mod app;
#[path = "core-renderer.rs"]
mod core_renderer;
mod instance;
mod surface;
mod device;

pub use app::RenderApp;


use instance::RenderInstance;
use surface::RenderSurface;
use device::RenderDevice;

pub struct HelloRenderer {
    instance: RenderInstance,

    surface: RenderSurface,

    device: RenderDevice,

}