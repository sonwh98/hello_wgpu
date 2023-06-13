fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("testing")
        .build(&event_loop)
        .unwrap();

    println!("window= {:?}", window);
}
