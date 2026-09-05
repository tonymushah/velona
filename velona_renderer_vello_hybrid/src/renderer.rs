use debug_timer::debug_timer;
use futures_channel::oneshot;
use imaging::{FillRef, GeometryRef, PaintSink};
use kurbo::Rect;
use peniko::{BrushRef, Color};
use pollster::FutureExt;
use std::sync::{Arc, RwLock};
use vello_hybrid::{RenderSettings, RenderTargetConfig, Scene as VelloHybridScene};
use velona_renderer::{WindowRenderer, window_handle::WindowHandle};
use wgpu::{CompositeAlphaMode, PresentMode, TextureFormat};
use wgpu_context::{DeviceHandle, SurfaceRenderer, SurfaceRendererConfiguration, WGPUContext};

use crate::imaging::{VelloHybridRenderer, VelloHybridSceneSink};

struct ActiveRenderState {
    renderer: VelloHybridRenderer,
    render_surface: SurfaceRenderer<'static>,
}

/// Result of a successful asynchronous resume; both the active state and the
/// `WGPUContext` are returned so the renderer can reclaim the context.
struct InitOutput {
    active: ActiveRenderState,
}

#[allow(clippy::large_enum_variant)]
enum RenderState {
    Suspended,
    Pending {
        receiver: oneshot::Receiver<InitOutput>,
    },
    Active(ActiveRenderState),
}

#[derive(Clone)]
#[non_exhaustive]
pub struct VelloHybridRendererOptions {
    pub render_settings: RenderSettings,
    pub base_color: Color,
    /// Alpha mode used when compositing the window surface.
    // pub composite_alpha_mode: velona_renderer::CompositeAlphaMode,
    /// Maximum number of frames the presentation engine may queue ahead of the
    /// display. Each queued frame requires a window-sized swapchain surface, so
    /// lower values reduce memory usage and input latency at the cost of less
    /// buffering to absorb slow frames.
    pub desired_maximum_frame_latency: u32,
}

impl Default for VelloHybridRendererOptions {
    fn default() -> Self {
        Self {
            render_settings: RenderSettings::default(),
            base_color: Color::WHITE,
            // composite_alpha_mode: anyrender::CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 1,
        }
    }
}

impl VelloHybridRendererOptions {
    pub fn new() -> Self {
        // Within default of RenderSettings there are calls to non const methods so no const for new
        Self::default()
    }

    pub const fn render_settings(self, render_settings: RenderSettings) -> Self {
        Self {
            render_settings,
            ..self
        }
    }

    pub const fn base_color(self, base_color: Color) -> Self {
        Self { base_color, ..self }
    }

    // pub const fn composite_alpha_mode(
    //     self,
    //     composite_alpha_mode: anyrender::CompositeAlphaMode,
    // ) -> Self {
    //     Self {
    //         composite_alpha_mode,
    //         ..self
    //     }
    // }

    pub const fn desired_maximum_frame_latency(self, desired_maximum_frame_latency: u32) -> Self {
        Self {
            desired_maximum_frame_latency,
            ..self
        }
    }
}

// impl From<anyrender::RendererConfig> for VelloHybridRendererOptions {
//     fn from(config: anyrender::RendererConfig) -> Self {
//         Self {
//             base_color: config.base_color.unwrap_or(Color::WHITE),
//             composite_alpha_mode: config.composite_alpha_mode.unwrap_or_default(),
//             ..Default::default()
//         }
//     }
// }

pub struct VelloHybridWindowRenderer {
    // The fields MUST be in this order, so that the surface is dropped before the window
    // Window is cached even when suspended so that it can be reused when the app is resumed after being suspended
    render_state: RenderState,
    window_handle: Option<Arc<dyn WindowHandle>>,

    wgpu_context: Arc<RwLock<WGPUContext>>,
    scene: VelloHybridScene,
    config: VelloHybridRendererOptions,
}
impl VelloHybridWindowRenderer {
    pub fn new(context: Arc<RwLock<WGPUContext>>) -> Self {
        Self::with_options(context, VelloHybridRendererOptions::default())
    }

    pub fn with_options(
        context: Arc<RwLock<WGPUContext>>,
        config: impl Into<VelloHybridRendererOptions>,
    ) -> Self {
        let config = config.into();
        let render_settings = config.render_settings;
        Self {
            render_state: RenderState::Suspended,
            config,
            wgpu_context: context,
            window_handle: None,
            scene: VelloHybridScene::new_with(0, 0, render_settings.level),
        }
    }

    pub fn current_device_handle(&self) -> Option<&DeviceHandle> {
        match &self.render_state {
            RenderState::Active(active) => Some(&active.render_surface.device_handle),
            _ => None,
        }
    }
}

// TODO: Make configurable?
#[cfg(target_os = "android")]
const DEFAULT_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
#[cfg(not(target_os = "android"))]
const DEFAULT_TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

impl WindowRenderer for VelloHybridWindowRenderer {
    type ScenePainter<'a>
        = VelloHybridSceneSink<'a>
    where
        Self: 'a;

    fn is_active(&self) -> bool {
        matches!(self.render_state, RenderState::Active { .. })
    }

    fn is_pending(&self) -> bool {
        matches!(self.render_state, RenderState::Pending { .. })
    }

    fn resume(&mut self, window_handle: Arc<dyn WindowHandle>, width: u32, height: u32) {
        // Each `resume` must be preceded by `suspend` (or be the first call after
        // construction). Calling while `Pending` or `Active` is a state-machine bug
        // in the embedder: it would orphan the in-flight init's `WGPUContext` and
        // pay for a fresh adapter+device init on the fallback path below.
        if !matches!(self.render_state, RenderState::Suspended) {
            // #[cfg(feature = "tracing")]
            // tracing::warn!("WindowRenderer::resume called from non-Suspended state");
            return;
        }

        let (sender, receiver) = oneshot::channel();
        self.render_state = RenderState::Pending { receiver };
        self.window_handle = Some(window_handle.clone());

        // Reset the scene to the new dimensions before init kicks off, so callers that
        // query scene size (e.g. `set_size`) see consistent state.
        let render_settings = self.config.render_settings;
        self.scene = VelloHybridScene::new_with(width as u16, height as u16, render_settings.level);

        let surface = self
            .wgpu_context
            .write()
            .unwrap()
            .create_surface(window_handle)
            .expect("Error creating surface");
        let instance = self.wgpu_context.write().unwrap().instance.clone();
        let extra_features = self.wgpu_context.read().unwrap().extra_features();
        let override_limits = self.wgpu_context.read().unwrap().override_limits();
        // let mut composite_alpha_mode = match self.config.composite_alpha_mode {
        //     anyrender::CompositeAlphaMode::Auto => CompositeAlphaMode::Auto,
        //     anyrender::CompositeAlphaMode::Opaque => CompositeAlphaMode::Opaque,
        //     anyrender::CompositeAlphaMode::Transparent => {
        //         #[cfg(target_vendor = "apple")]
        //         {
        //             // wgpu is lying in apple's case it uses PreMultiplied in reality
        //             // (do not modify shaders for PostMultiplied)
        //             CompositeAlphaMode::PostMultiplied
        //         }
        //         #[cfg(not(target_vendor = "apple"))]
        //         {
        //             CompositeAlphaMode::PreMultiplied
        //         }
        //     }
        // };
        let mut composite_alpha_mode = CompositeAlphaMode::Auto;
        let desired_maximum_frame_latency = self.config.desired_maximum_frame_latency;
        let existing_device_handle = self
            .wgpu_context
            .write()
            .unwrap()
            .find_compatible_device_handle(Some(&surface));

        let device_handle = match existing_device_handle {
            Some(device_handle) => device_handle,
            None => DeviceHandle::new_from_compatible_surface(
                instance,
                Some(&surface),
                extra_features,
                override_limits,
            )
            .block_on()
            .expect("Error creating DeviceHandle"),
        };

        let adapter = &device_handle.adapter;
        let caps = surface.get_capabilities(adapter);
        let mut alpha_modes = caps.alpha_modes;

        if !alpha_modes.contains(&composite_alpha_mode) {
            alpha_modes.sort_unstable_by(
                |first: &CompositeAlphaMode, second: &CompositeAlphaMode| {
                    let first_num = match *first {
                        CompositeAlphaMode::PreMultiplied => 0,
                        CompositeAlphaMode::PostMultiplied => 1,
                        CompositeAlphaMode::Opaque
                        | CompositeAlphaMode::Inherit
                        | CompositeAlphaMode::Auto => 2,
                    };
                    let second_num = match *second {
                        CompositeAlphaMode::PreMultiplied => 0,
                        CompositeAlphaMode::PostMultiplied => 1,
                        CompositeAlphaMode::Opaque
                        | CompositeAlphaMode::Inherit
                        | CompositeAlphaMode::Auto => 2,
                    };
                    first_num.cmp(&second_num)
                },
            );
            composite_alpha_mode = alpha_modes
                .first()
                .copied()
                .expect("Surface didn't report any alpha modes");
        }

        // Vello Hybrid emits premultiplied alpha and renders directly into
        // its target. That matches a `PreMultiplied` surface (and Opaque
        // ignores alpha), so those render straight to the surface. Only
        // `PostMultiplied` (straight alpha) needs conversion, which routes
        // through an intermediate texture (using the renderer's target
        // format) that is un-premultiplied while blitting to the surface.
        #[cfg(not(target_vendor = "apple"))]
        let intermediate_texture = (composite_alpha_mode == CompositeAlphaMode::PostMultiplied)
            .then(|| {
                use wgpu::TextureUsages;
                use wgpu_context::{AlphaConversion, TextureConfiguration};
                TextureConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    format: DEFAULT_TEXTURE_FORMAT,
                    alpha_conversion: Some(AlphaConversion::Unpremultiply),
                }
            });

        // Apple is almost guaranteed to be premultiplied
        // TODO: Remove below once gfx-rs/wgpu#9896 gets fixed
        #[cfg(target_vendor = "apple")]
        let intermediate_texture = None;

        let render_surface = SurfaceRenderer::new(
            surface,
            SurfaceRendererConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                formats: vec![DEFAULT_TEXTURE_FORMAT],
                width,
                height,
                present_mode: PresentMode::AutoVsync,
                desired_maximum_frame_latency,
                alpha_mode: composite_alpha_mode,
                view_formats: vec![],
            },
            intermediate_texture,
            device_handle,
        )
        .expect("Error creating SurfaceRenderer");

        let renderer = VelloHybridRenderer::new_with_config(
            render_surface.device().clone(),
            render_surface.queue().clone(),
            &RenderTargetConfig {
                format: DEFAULT_TEXTURE_FORMAT,
                width,
                height,
            },
            self.config.render_settings,
        );

        let _ = sender.send(InitOutput {
            active: ActiveRenderState {
                renderer,
                render_surface,
            },
        });
    }

    fn complete_resume(&mut self) -> bool {
        match &mut self.render_state {
            RenderState::Active { .. } => true,
            RenderState::Suspended => false,
            RenderState::Pending { receiver } => match receiver.try_recv() {
                Ok(Some(InitOutput { active })) => {
                    let device_handle = active.render_surface.device_handle.clone();
                    self.wgpu_context
                        .write()
                        .unwrap()
                        .device_pool
                        .push(device_handle);
                    self.render_state = RenderState::Active(active);
                    true
                }
                _ => false,
            },
        }
    }

    fn suspend(&mut self) {
        self.render_state = RenderState::Suspended;
    }

    fn set_size(&mut self, width: u32, height: u32) {
        if width as u16 != self.scene.width() || height as u16 != self.scene.height() {
            self.scene.reset_and_resize(width as u16, height as u16);
            if let RenderState::Active(active) = &mut self.render_state {
                active.render_surface.resize(width, height);
            };
        }
    }

    fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(&mut self, draw_fn: F) {
        let RenderState::Active(state) = &mut self.render_state else {
            return;
        };

        let render_surface = &mut state.render_surface;

        debug_timer!(timer, feature = "log_frame_times");

        // let mut encoder =
        //     render_surface
        //         .device()
        //         .create_command_encoder(&CommandEncoderDescriptor {
        //             label: Some("Render scene"),
        //         });

        let mut scene_sink =
            VelloHybridSceneSink::with_renderer(&mut self.scene, &mut state.renderer);
        if self.config.base_color != Color::TRANSPARENT {
            scene_sink.fill(
                FillRef::new(
                    GeometryRef::Rect(Rect::new(
                        0.,
                        0.,
                        render_surface.config.width.into(),
                        render_surface.config.height.into(),
                    )),
                    BrushRef::Solid(self.config.base_color),
                )
                .fill_rule(peniko::Fill::NonZero)
                .brush_transform(None)
                .transform(kurbo::Affine::IDENTITY),
            );
        }
        // if self.config.base_color != Color::TRANSPARENT {
        //     scene_sink.fill(
        //         FillRef::new(shape, brush)
        //         peniko::Fill::NonZero,
        //         kurbo::Affine::IDENTITY,
        //         self.config.base_color,
        //         None,
        //         &kurbo::Rect::new(
        //             0.,
        //             0.,
        //             render_surface.config.width as f64,
        //             render_surface.config.height as f64,
        //         ),
        //     );
        // }
        // Regenerate the vello scene
        draw_fn(&mut scene_sink);
        timer.record_time("cmd");

        let Ok(texture_view) = render_surface.target_texture_view() else {
            // Skip frame in case of error getting surface texture
            render_surface.clear_surface_texture();
            return;
        };

        // Construct Vello Hybrid TextureBindings
        // let mut texture_bindings = TextureBindings::new();
        // for (resource_id, texture_view) in state.texture_bindings.iter() {
        //     texture_bindings.insert(TextureId(resource_id.into_ffi()), texture_view.clone());
        // }

        state
            .renderer
            .render_to_texture_view(
                &self.scene,
                &texture_view,
                render_surface.config.width,
                render_surface.config.height,
            )
            .expect("failed to render to texture");
        timer.record_time("render");

        drop(texture_view);

        if render_surface.maybe_blit_and_present().is_err() {
            return;
        }
        timer.record_time("present");

        render_surface
            .device()
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        timer.record_time("wait");
        timer.print_times("vello_hybrid: ");

        // Empty the Vello scene (memory optimisation)
        self.scene.reset();
        // state.renderer.clear_cached_images();
    }
}
