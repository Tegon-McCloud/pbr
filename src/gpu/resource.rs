
use ash::{vk, prelude::VkResult};

use super::context::Context;

pub struct Buffer<'c> {
    ctx: &'c Context,
    raw: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl<'c> Drop for Buffer<'c> {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_buffer(self.raw, None);
            self.ctx.device.free_memory(self.memory, None);
        }
    }
}

impl<'c> Buffer<'c> {
    pub fn new(ctx: &'c Context, size: u64) -> VkResult<Buffer<'c>> {
        unsafe {
            let create_info = vk::BufferCreateInfo::builder()
            .flags(vk::BufferCreateFlags::empty())
            .size(size)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(std::slice::from_ref(&ctx.queue_family));

            let buffer = ctx.device.create_buffer(&create_info, None)?;
            let mem_reqs = ctx.device.get_buffer_memory_requirements(buffer);
            let mem_type = ctx.pick_memory_type(&mem_reqs).expect("no suitable memory type found");
            
            let alloc_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(size)
                .memory_type_index(mem_type);

            let memory = ctx.device.allocate_memory(&alloc_info, None)?;
            
            Ok(Self { ctx, raw: buffer, memory })
        }
    }


}

pub struct Texture {
    raw: vk::Image,
    memory: vk::DeviceMemory,
}