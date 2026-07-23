#[allow(
    unsafe_op_in_unsafe_fn,
)]

use std::borrow::Cow;
use std::error::Error;
use std::ffi::{c_char, CStr};
use ash::{vk, Instance};
use ash::Entry;
use ash::ext::debug_utils;
use raw_window_handle::HasDisplayHandle;

const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

pub struct RenderInstance {
    pub entry: Entry,
    pub instance: Instance,
    debug_callback: vk::DebugUtilsMessengerEXT,
    debug_utils_loader: debug_utils::Instance,
}

impl RenderInstance {
    pub fn new(window: &winit::window::Window) -> Result<Self, Box<dyn Error>> {
        let entry = Entry::linked();
        let application_name = c"My App";
        let engine_name = c"Hello ECS";
        let mut layer_names : &[&CStr] = &[];
        #[cfg(debug_assertions)]{
            layer_names  = &[VALIDATION_LAYER];
        }
        
        let layers_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())?.to_vec();
        extension_names.push(debug_utils::NAME.as_ptr());


        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let appinfo = vk::ApplicationInfo::default()
            .application_name(application_name)
            .application_version(0)
            .engine_name(engine_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);


        let instance: Instance = unsafe {
            entry
                .create_instance(&create_info, None)
        }.expect("Instance creation error");

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                ,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
        let debug_callback = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
        }?;

        Ok(Self{
            entry,
            instance,
            debug_callback,
            debug_utils_loader,
        })
    }
}

impl Drop for RenderInstance {
    fn drop(&mut self) {
        unsafe { self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_callback, None); }
        unsafe { self.instance.destroy_instance(None) }
    }
}



unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 { unsafe {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}}