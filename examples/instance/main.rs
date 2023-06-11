fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(), //wgpu::Backends::METAL,
        dx12_shader_compiler: Default::default(),
    });

    println!("instance= {:?}", instance);
}
