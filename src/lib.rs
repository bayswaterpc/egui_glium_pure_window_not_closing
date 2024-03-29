//! Example how to use pure `egui_glium` without [`epi`].
use glium::backend::glutin::glutin::platform::windows::WindowExtWindows;
use glium::glutin;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use winapi::{shared::windef::HWND, um::winuser};

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glium example");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, &event_loop).unwrap()
}

fn redraw(
    egui: &mut egui_glium::EguiGlium,
    display: &glium::Display,
    control_flow: &mut ControlFlow,
) {
    egui.begin_frame(&display);

    let mut quit = false;

    egui::SidePanel::left("my_side_panel", 300.0).show(egui.ctx(), |ui| {
        ui.heading("Hello World!");
        if ui.button("Quit").clicked() {
            quit = true;
        }

        egui::ComboBox::from_label("Version")
            .width(150.0)
            .selected_text("foo")
            .show_ui(ui, |ui| {
                egui::CollapsingHeader::new("Dev")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label("contents");
                    });
            });
    });

    let (needs_repaint, shapes) = egui.end_frame(&display);

    *control_flow = if quit {
        glutin::event_loop::ControlFlow::Exit
    } else if needs_repaint {
        display.gl_window().window().request_redraw();
        glutin::event_loop::ControlFlow::Poll
    } else {
        glutin::event_loop::ControlFlow::Wait
    };

    {
        use glium::Surface as _;
        let mut target = display.draw();

        let clear_color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
        target.clear_color(
            clear_color[0],
            clear_color[1],
            clear_color[2],
            clear_color[3],
        );

        // draw things behind egui here

        egui.paint(&display, &mut target, shapes);

        // draw things on top of egui here

        target.finish().unwrap();
    }
}

pub fn egui_glium_pure_example() {
    let mut event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&&event_loop);

    let mut egui = egui_glium::EguiGlium::new(&display);
    let h_wnd = display.gl_window().window().hwnd() as HWND;

    event_loop.run_return(move |event, _, control_flow| {
        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => {
                redraw(&mut egui, &display, control_flow)
            }
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => {
                redraw(&mut egui, &display, control_flow)
            }

            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(event, control_flow);
                display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            _ => (),
        }
    });

    //"Explicit call to DestroyWindow() shouldn't be necessary but isn't closing otherwise");
    // unsafe {
    //     winuser::DestroyWindow(h_wnd);
    // }
}
