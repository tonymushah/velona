use std::{
    collections::{BTreeMap, HashMap},
    num::NonZeroUsize,
    sync::Arc,
};

use log::warn;
use masonry::{
    app::{RenderRoot, RenderRootOptions},
    core::{DefaultProperties, NewWidget, Widget},
    vello::{
        self, RendererOptions,
        wgpu::{self, InstanceDescriptor},
    },
    widgets::{Flex, ZStack},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window as WinitWindow, WindowAttributes, WindowId},
};

pub struct Window {
    pub(crate) winit_window: Arc<WinitWindow>,
    render_root: RenderRoot,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
}

impl Window {
    async fn new<V>(
        window: WinitWindow,
        instance: &wgpu::Instance,
        view: V,
        default_properties: Arc<DefaultProperties>,
    ) -> Result<Self, crate::error::Error>
    where
        V: FnOnce() -> NewWidget<dyn Widget>,
    {
        let window = Arc::new(window);
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        log::info!("adapter info: {:#?}", adapter);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let render_root = RenderRoot::new(
            view(),
            |_| {},
            RenderRootOptions {
                default_properties,
                use_system_fonts: true,
                size_policy: masonry::app::WindowSizePolicy::User,
                size,
                scale_factor: 1.0,
                test_font: None,
            },
        );
        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            winit_window: window,
            render_root,
        })
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }
    pub fn render(&mut self) -> Result<(), crate::error::Error> {
        self.winit_window.request_redraw();
        if !self.is_surface_configured {
            return Ok(());
        }
        let mut renderer = vello::Renderer::new(
            &self.device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )?;

        let (scene, _access_tree) = self.render_root.redraw();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        renderer.render_to_texture(
            &self.device,
            &self.queue,
            &scene,
            &view,
            &vello::RenderParams {
                base_color: masonry::palette::css::BLACK, // Background color
                width: self.config.width,
                height: self.config.height,
                antialiasing_method: vello::AaConfig::Msaa16,
            },
        )?;
        output.present();

        Ok(())
    }
}

struct App {
    event_loop_proxy: EventLoopProxy<()>,
    windows: HashMap<WindowId, Box<Window>>,
    instance: wgpu::Instance,
    default_properties: Arc<DefaultProperties>,
}

#[derive(Default)]
pub struct Builder {
    event_loop_builder: EventLoopBuilder<()>,
    instance_descriptor: Option<InstanceDescriptor>,
    default_properties: DefaultProperties,
}

impl Builder {
    pub fn run(mut self) -> Result<(), crate::error::Error> {
        let event_loop = self.event_loop_builder.build()?;
        let proxy = event_loop.create_proxy();
        let instance_descriptor = self.instance_descriptor.unwrap_or(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY.union(wgpu::Backends::SECONDARY),
            ..Default::default()
        });
        let mut app = App {
            event_loop_proxy: proxy,
            windows: Default::default(),
            instance: wgpu::Instance::new(&instance_descriptor),
            default_properties: Arc::new(self.default_properties),
        };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match event_loop.create_window(WindowAttributes::default()) {
            Ok(window) => {
                match pollster::block_on(Window::new(
                    window,
                    &self.instance,
                    || NewWidget::new(ZStack::new()).erased(),
                    self.default_properties.clone(),
                )) {
                    Ok(new_instance) => {
                        self.windows
                            .insert(new_instance.winit_window.id(), Box::new(new_instance));
                    }
                    Err(err) => {
                        log::error!("Cannot create new window ({err})")
                    }
                }
            }
            Err(err) => {
                log::error!("Os error on creating new window {err}");
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.windows.contains_key(&window_id) {
            match event {
                WindowEvent::Destroyed => {
                    self.windows.remove(&window_id);
                    if self.windows.is_empty() {
                        event_loop.exit();
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Some(win) = self.windows.get_mut(&window_id) {
                        match win.render() {
                            Ok(_) => {}
                            Err(crate::error::Error::Surface(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            )) => {
                                let size = win.winit_window.inner_size();
                                win.resize(size.width, size.height);
                            }
                            Err(e) => {
                                log::error!("Unable to render {}", e);
                            }
                        }
                    } else {
                        warn!("No matching window state found for {:?}", window_id);
                    }
                }
                _ => {}
            }
        }
    }
    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.windows.shrink_to_fit();
    }
}
