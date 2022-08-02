use ash::{prelude::VkResult, vk, extensions::ext};

pub struct Context {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    
    pub debug_utils_loader: ext::DebugUtils,
    pub debug_messenger: vk::DebugUtilsMessengerEXT,

    pub physical_device: vk::PhysicalDevice,
    pub queue_family: u32,
    pub device: ash::Device,
    pub queue: vk::Queue,
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

impl Context {
    pub unsafe fn new() -> VkResult<Self> {
        let entry = ash::Entry::load().expect("Failed to initialize ash");
        let instance = Self::create_instance(&entry)?;

        let debug_utils_loader = ext::DebugUtils::new(&entry, &instance);
        let debug_messenger = Self::create_debug_messenger(&debug_utils_loader)?;

        let (physical_device, queue_family) = Self::pick_adapter_and_queue_family(&instance)?.expect("No suitable physical device was found.");
        let (device, queue) = Self::create_device_and_queue(&instance, physical_device, queue_family)?;

        Ok(Self {
            entry,
            instance,

            debug_utils_loader,
            debug_messenger,

            physical_device,
            queue_family,
            device,
            queue,
        })
    }

    pub fn pick_memory_type(&self, reqs: &vk::MemoryRequirements) -> Option<u32> {
        
        // Safety: it is invariant for Context to have a valid physical device
        let props = unsafe { self.instance.get_physical_device_memory_properties(self.physical_device) };

        props.memory_types[..props.memory_type_count as usize].iter()
            .enumerate()
            .filter(|(i, _)| (reqs.memory_type_bits & (1 << i)) != 0)
            .map(|(i, _)| i as u32)
            .next()
    }

}


impl Context {

    unsafe fn create_instance(entry: &ash::Entry) -> VkResult<ash::Instance> {
        let engine_name = std::ffi::CString::new("An Engine").unwrap();
        let app_name = std::ffi::CString::new("An App").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 1))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1,1))
            .api_version(vk::API_VERSION_1_2);
    
        let layer_names = vec![
            std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap(),
        ];
        let layer_name_pointers = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect::<Vec<_>>();


        let extension_name_pointers: Vec<*const i8> = vec![
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];
        

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_name_pointers)
            .enabled_extension_names(&extension_name_pointers);
            
        entry.create_instance(&create_info, None)
    } 

    unsafe fn create_debug_messenger(loader: &ext::DebugUtils) -> VkResult<vk::DebugUtilsMessengerEXT> {

        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION)
            .pfn_user_callback(Some(debug_utils_callback));

        loader.create_debug_utils_messenger(&create_info, None)
    }

    unsafe fn pick_adapter_and_queue_family(instance: &ash::Instance) -> VkResult<Option<(vk::PhysicalDevice, u32)>> {
        Ok(instance.enumerate_physical_devices()?.iter()
            .filter_map(|&pd| Self::pick_queue_family(instance, pd).and_then(|qf| Some((pd, qf))))
            .next())
    }

    unsafe fn pick_queue_family(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Option<u32> {
        instance.get_physical_device_queue_family_properties(physical_device).iter()
            .enumerate()
            .filter(|(_, props)| props.queue_flags.contains(vk::QueueFlags::COMPUTE))
            .map(|(i, _)| i as u32)
            .next()
    }

    unsafe fn create_device_and_queue(instance: &ash::Instance, physical_device: vk::PhysicalDevice, queue_family: u32) -> VkResult<(ash::Device, vk::Queue)> {
        
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family)
            .queue_priorities(&[1.0]);

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_create_info));
        
        let device = instance.create_device(physical_device, &create_info, None)?;

        let queue = device.get_device_queue(queue_family, 0);
        
        Ok((device, queue))
    }
}

unsafe extern "system" fn debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {

    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message).to_string_lossy();
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();

    if message.starts_with("loaderAddLayerProperties")  ||
        message.contains("Failed to find 'vkGetInstanceProcAddr' in layer") {
        return vk::FALSE
    }
    
    if message_severity.intersects(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        eprintln!("[debug][{}][{}] {}\n", severity, ty, message);
    } 
    
    if message_severity.intersects(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE | vk::DebugUtilsMessageSeverityFlagsEXT::INFO){
        //println!("[debug][{}][{}] {}\n", severity, ty, message);
    }

    vk::FALSE
}