use std::{
    collections::{BTreeMap, HashMap},
    num::NonZeroUsize,
    sync::Arc,
};

use log::warn;
use masonry::{
    app::{RenderRoot, RenderRootOptions},
    core::{DefaultProperties, NewWidget, Properties, Widget},
    palette::css,
    properties::ContentColor,
    vello::{
        self, RendererOptions,
        util::RenderSurface,
        wgpu::{self, InstanceDescriptor, util::TextureBlitter, wgt::TextureDescriptor},
    },
    widgets::{ChildAlignment, Flex, ZStack},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
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
    renderer: vello::Renderer,
    blitter: TextureBlitter,
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

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|it| {
                matches!(
                    it,
                    wgpu::TextureFormat::Rgba8Unorm
                        | wgpu::TextureFormat::Bgra8Unorm
                        | wgpu::TextureFormat::Bgra8UnormSrgb
                )
            })
            .copied()
            .ok_or(crate::error::Error::UnsupportedSurfaceFormat)?;
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
            blitter: TextureBlitter::new(&device, surface_format),
            renderer: vello::Renderer::new(
                &device,
                RendererOptions {
                    use_cpu: false,
                    antialiasing_support: vello::AaSupport::all(),
                    num_init_threads: NonZeroUsize::new(1),
                    pipeline_cache: None,
                },
            )?,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            winit_window: window,
            render_root,
        })
    }
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.render_root
                .handle_window_event(masonry::core::WindowEvent::Resize(size));
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }
    fn render_scene(&mut self, scene: &vello::Scene) -> Result<(), crate::error::Error> {
        let scene_texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Vello scene render"),
            size: {
                wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                }
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Rgba8Unorm,
            view_formats: &[],
        });
        let scene_texture_view = scene_texture.create_view(&Default::default());

        let mut renderer = vello::Renderer::new(
            &self.device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )?;
        renderer.render_to_texture(
            &self.device,
            &self.queue,
            scene,
            &scene_texture_view,
            &vello::RenderParams {
                base_color: masonry::palette::css::BLACK, // Background color
                width: self.config.width,
                height: self.config.height,
                antialiasing_method: vello::AaConfig::Area,
            },
        )?;
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surface Blit"),
            });
        self.blitter
            .copy(&self.device, &mut encoder, &scene_texture_view, &view);
        self.queue.submit([encoder.finish()]);
        self.winit_window.pre_present_notify();

        output.present();
        Ok(())
    }
    pub fn render(&mut self) -> Result<(), crate::error::Error> {
        // self.winit_window.request_redraw();
        if !self.is_surface_configured {
            return Ok(());
        }
        let (scene, _access_tree) = self.render_root.redraw();
        let scale_factor = self.winit_window.scale_factor();

        let transformed_scene = if scale_factor == 1.0 {
            None
        } else {
            let mut new_scene = vello::Scene::new();
            new_scene.append(
                &scene,
                Some(masonry::vello::kurbo::Affine::scale(scale_factor)),
            );
            Some(new_scene)
        };
        let scene_ref = transformed_scene.as_ref().unwrap_or(&scene);

        self.render_scene(scene_ref)?;

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

impl App {
    fn handle_redraw_request(&mut self, window_id: WindowId) {
        if let Some(win) = self.windows.get_mut(&window_id) {
            match win.render() {
                Ok(_) => {}
                Err(crate::error::Error::Surface(
                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                )) => {
                    let size = win.winit_window.inner_size();
                    win.resize(size);
                }
                Err(e) => {
                    log::error!("Unable to render {}", e);
                }
            }
        } else {
            warn!("No matching window state found for {:?}", window_id);
        }
    }
    fn handle_resize_event(&mut self, window_id: WindowId, size: PhysicalSize<u32>) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.resize(size);
        } else {
            warn!("No window found for resizing");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = non_exhaustive::non_exhaustive!(WindowAttributes {
            visible: true,
            inner_size: Some(winit::dpi::Size::Physical(PhysicalSize {
                width: 500,
                height: 300,
            })),
        });
        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                match pollster::block_on(Window::new(
                    window,
                    &self.instance,
                    || {
                        ZStack::new()
                            .with_child(
                                masonry::widgets::Label::new("aaaaaaa")
                                    .with_props(Properties::one(ContentColor::new(css::BEIGE))),
                                ChildAlignment::ParentAligned,
                            )
                            .with_auto_id()
                            .erased()
                    },
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
        match event {
            WindowEvent::RedrawRequested => {
                self.handle_redraw_request(window_id);
            }
            WindowEvent::Resized(size) => {
                self.handle_resize_event(window_id, size);
            }
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.windows.shrink_to_fit();
    }
}
