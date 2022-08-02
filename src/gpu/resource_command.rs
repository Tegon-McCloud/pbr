
enum ResourceCommand {
    CreateBuffer {
        
    }

}

pub struct ResourceCommandBuffer {
    cmds: Vec<ResourceCommand>,
}
