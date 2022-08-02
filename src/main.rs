
pub mod gpu;

fn main() {
    unsafe {

        let ctx = gpu::context::Context::new().unwrap();

        let _buf = gpu::resource::Buffer::new(&ctx, 4096);   
         
    }

    

}
