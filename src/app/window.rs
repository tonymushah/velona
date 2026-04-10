use std::{num::NonZeroUsize, sync::Arc, time::Instant};

use masonry::{
    app::RenderRootOptions,
    core::{DefaultProperties, NewWidget, Widget},
    vello::{
        self, RendererOptions,
        wgpu::{self, util::TextureBlitter, wgt::TextureDescriptor},
    },
};
use reactive_graph::owner::{Owner, provide_context};
use send_wrapper::SendWrapper;
use ui_events_winit::WindowEventReducer;
use winit::window::Window as WinitWindow;

use crate::{
    app::{
        AppEventLoopProxy,
        el_event::{
            RenderRootNewLayer, RenderRootRemoveLayer, RenderRootRepositionLayer, WidgetAction,
        },
    },
    convert_winit_event::masonry_resize_direction_to_winit,
    render_root::{InnerRenderRoot, WindowRenderRoot},
    utils::todo_warn_of_something,
    window_event_handler::InternWindowEventHandler,
};

pub struct Window {
    pub(crate) winit_window: Arc<WinitWindow>,
    pub(crate) render_root: WindowRenderRoot,
    surface: Option<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: vello::Renderer,
    blitter: TextureBlitter,
    pub(crate) access_kit: accesskit_winit::Adapter,
    owner: Owner,
    pub(crate) event_reducer: WindowEventReducer,
    // Is `Some` if the most recently displayed frame was an animation frame.
    last_anim: Option<Instant>,
    pub(crate) window_event_handler: InternWindowEventHandler,
}

pub struct WindowNew<'i, V> {
    pub window: Arc<WinitWindow>,
    pub instance: &'i wgpu::Instance,
    pub view: V,
    pub default_properties: Arc<DefaultProperties>,
    pub access_kit: accesskit_winit::Adapter,
    pub event_loop_proxy: AppEventLoopProxy,
    pub parent_owner: &'i Owner,
}

impl Drop for Window {
    fn drop(&mut self) {
        self.owner.cleanup();
    }
}

impl Window {
    pub fn on_memory_warning(&mut self) {
        self.render_root.use_inner_render_root_ref(|rr| {
            let mut write = self.window_event_handler.write();
            write.cleanup(&rr.tree);
            write.shrink_to_fit();
        });
    }
    pub(crate) async fn new<V>(args: WindowNew<'_, V>) -> Result<Self, crate::error::Error>
    where
        V: FnOnce() -> NewWidget<dyn Widget>,
    {
        let WindowNew {
            window,
            instance,
            view,
            default_properties,
            access_kit,
            event_loop_proxy,
            parent_owner,
        } = args;
        let window_owner = parent_owner.child();
        let event_handlers = InternWindowEventHandler::default();

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone())?;

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
            .find(|it| it.is_srgb())
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

        let renderer = vello::Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )?;

        let blitter = TextureBlitter::new(&device, surface_format);

        let render_root = InnerRenderRoot::new(
            {
                let window = window.clone();
                move |ev| match ev {
                    masonry::app::RenderRootSignal::Action(any_debug, widget_id) => {
                        let _ = event_loop_proxy.send_event(
                            WidgetAction {
                                window_id: window.id(),
                                event: SendWrapper::new(any_debug),
                                widget_id,
                            }
                            .into(),
                        );
                    }
                    masonry::app::RenderRootSignal::StartIme => {
                        window.set_ime_allowed(true);
                    }
                    masonry::app::RenderRootSignal::EndIme => {
                        window.set_ime_allowed(false);
                    }
                    masonry::app::RenderRootSignal::ImeMoved(_logical_position, _logical_size) => {
                        window.set_ime_cursor_area(_logical_position, _logical_size);
                    }
                    masonry::app::RenderRootSignal::ClipboardStore(_) => {
                        todo_warn_of_something("RenderRootSignal::ClipboardStore");
                    }
                    masonry::app::RenderRootSignal::RequestRedraw => {
                        window.request_redraw();
                    }
                    masonry::app::RenderRootSignal::RequestAnimFrame => {
                        window.request_redraw();
                    }
                    masonry::app::RenderRootSignal::TakeFocus => {
                        window.focus_window();
                    }
                    masonry::app::RenderRootSignal::SetCursor(cursor_icon) => {
                        window.set_cursor(cursor_icon);
                    }
                    masonry::app::RenderRootSignal::SetSize(physical_size) => {
                        let _ = window.request_inner_size(physical_size);
                    }
                    masonry::app::RenderRootSignal::SetTitle(title) => {
                        window.set_title(&title);
                    }
                    masonry::app::RenderRootSignal::DragWindow => {
                        if let Err(err) = window.drag_window() {
                            log::warn!("Cannot draw window ({})", err);
                        }
                    }
                    masonry::app::RenderRootSignal::DragResizeWindow(resize_direction) => {
                        if let Err(err) = window
                            .drag_resize_window(masonry_resize_direction_to_winit(resize_direction))
                        {
                            log::warn!("Cannot drag resize window ({})", err);
                        }
                    }
                    masonry::app::RenderRootSignal::ToggleMaximized => {
                        window.set_maximized(!window.is_maximized());
                    }
                    masonry::app::RenderRootSignal::Minimize => {
                        window.set_minimized(true);
                    }
                    masonry::app::RenderRootSignal::Exit => {
                        todo_warn_of_something("RenderRootSignal::Exit");
                    }
                    masonry::app::RenderRootSignal::ShowWindowMenu(logical_position) => {
                        window.show_window_menu(logical_position);
                    }
                    masonry::app::RenderRootSignal::WidgetSelectedInInspector(_widget_id) => {
                        todo_warn_of_something("RenderRootSignal::WidgetSelectedInInspector");
                    }
                    masonry::app::RenderRootSignal::NewLayer(_new_widget, _point) => {
                        let _ = event_loop_proxy.send_event(
                            RenderRootNewLayer {
                                window_id: window.id(),
                                layer: _new_widget.into(),
                                point: _point,
                            }
                            .into(),
                        );
                    }
                    masonry::app::RenderRootSignal::RemoveLayer(_widget_id) => {
                        let _ = event_loop_proxy.send_event(
                            RenderRootRemoveLayer {
                                widget_id: _widget_id,
                                window_id: window.id(),
                            }
                            .into(),
                        );
                    }
                    masonry::app::RenderRootSignal::RepositionLayer(_widget_id, _point) => {
                        let _ = event_loop_proxy.send_event(
                            RenderRootRepositionLayer {
                                widget_id: _widget_id,
                                point: _point,
                                window_id: window.id(),
                            }
                            .into(),
                        );
                    }
                }
            },
            RenderRootOptions {
                default_properties,
                use_system_fonts: true,
                size_policy: masonry::app::WindowSizePolicy::User,
                size,
                scale_factor: window.scale_factor(),
                test_font: None,
            },
        );
        let render_root = WindowRenderRoot::new(render_root);
        {
            let new_widget = window_owner.with(|| {
                provide_context(render_root.create_weak());
                provide_context(event_handlers.get_weak());
                view()
            });
            if render_root
                .use_inner_render_root_mut(|root| {
                    root.swap_root_widget(new_widget);
                })
                .is_none()
            {
                log::error!("The render root should have been initialized already");
            }
        }

        Ok(Self {
            blitter,
            renderer,
            surface: Some(surface),
            device,
            queue,
            config,
            winit_window: window,
            render_root,
            owner: window_owner,
            access_kit,
            event_reducer: WindowEventReducer::default(),
            last_anim: None,
            window_event_handler: event_handlers,
        })
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

        self.renderer.render_to_texture(
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
        if let Some(surface) = self.surface.as_ref() {
            let output = surface.get_current_texture()?;
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
        }
        Ok(())
    }
    fn sync_surface_render_root_size(&mut self) -> bool {
        let Some(size) = self
            .render_root
            .use_inner_render_root_ref(|root| root.tree.size())
        else {
            return false;
        };
        self.config.width = size.width;
        self.config.height = size.height;
        if let Some(surface) = self.surface.as_mut() {
            surface.configure(&self.device, &self.config);
        }
        true
    }
    pub fn render(&mut self) -> Result<(), crate::error::Error> {
        if self.surface.is_none() {
            return Ok(());
        }

        let now = Instant::now();
        // TODO: this calculation uses wall-clock time of the paint call, which
        // potentially has jitter.
        //
        // See https://github.com/linebender/druid/issues/85 for discussion.
        let last = self.last_anim.take();
        let elapsed = last.map(|t| now.duration_since(t)).unwrap_or_default();

        self.render_root.use_inner_render_root_mut(|rr| {
            rr.tree
                .handle_window_event(masonry::core::WindowEvent::AnimFrame(elapsed));
        });

        // If this animation will continue, store the time.
        // If a new animation starts, then it will have zero reported elapsed time.
        let animation_continues = self
            .render_root
            .use_inner_render_root_ref(|rr| rr.tree.needs_anim())
            .unwrap_or_default();
        self.last_anim = animation_continues.then_some(now);

        let Some((scene, _access_tree)) = self
            .render_root
            .use_inner_render_root_mut(|root| root.tree.redraw())
        else {
            return Ok(());
        };

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

        if self.sync_surface_render_root_size() {
            self.render_scene(scene_ref)?;
        }
        if let Some(access_tree) = _access_tree {
            self.access_kit.update_if_active(|| access_tree);
        }
        Ok(())
    }
}
