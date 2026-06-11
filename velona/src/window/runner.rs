use std::{
    sync::{Arc, mpsc},
    time::Instant,
};

use imaging::RenderSource;
use masonry::{
    app::{RenderRootOptions, RenderRootSignal, VisualLayerKind},
    core::{DefaultProperties, NewWidget, Widget},
    palette::css::BLACK,
    peniko::color::{AlphaColor, Srgb},
};
use masonry_imaging::{Layer as ImagingLayer, PreparedFrame};
use reactive_graph::owner::{Owner, provide_context};
use ui_events_winit::WindowEventReducer;
use velona_renderer::WindowRenderer;
use winit::window::{Window as WinitWindow, WindowId};

use crate::{
    app::{AppHandle, el_event::EventProxyHandle},
    render_root::{InnerRenderRoot, WindowRenderRoot},
    window::{handle::WindowHandle, renderer::WindowRendererFactory},
    window_event_handler::InternWindowEventHandler,
};

pub struct Window<W>
where
    W: WindowRenderer,
{
    pub(crate) render_root: WindowRenderRoot,
    renderer: W,
    pub(crate) access_kit: accesskit_winit::Adapter,
    owner: Owner,
    pub(crate) event_reducer: WindowEventReducer,
    // Is `Some` if the most recently displayed frame was an animation frame.
    last_anim: Option<Instant>,
    pub(crate) window_event_handler: InternWindowEventHandler,
    base_color: AlphaColor<Srgb>,
    pub(crate) winit_window: Arc<WinitWindow>,
    handle: WindowHandle,
}

pub struct WindowNew<'i, V, W> {
    pub window: Arc<WinitWindow>,
    pub view: V,
    pub default_properties: Arc<DefaultProperties>,
    pub access_kit: accesskit_winit::Adapter,
    #[allow(unused)]
    pub app_handle: AppHandle,
    pub signal_sender: mpsc::Sender<(WindowId, RenderRootSignal)>,
    pub parent_owner: &'i Owner,
    pub base_color: Option<AlphaColor<Srgb>>,
    pub factory: &'i mut dyn WindowRendererFactory<WindowRenderer = W>,
}

impl<'i, V, W> WindowNew<'i, V, W> {
    pub fn handle(&self) -> WindowHandle {
        WindowHandle {
            window: Arc::downgrade(&self.window),
            app_handle: self.app_handle.clone(),
        }
    }
}

impl<W> Drop for Window<W>
where
    W: WindowRenderer,
{
    fn drop(&mut self) {
        self.owner.cleanup();
    }
}

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<W> Window<W>
where
    W: WindowRenderer,
{
    pub fn on_memory_warning(&mut self) {
        self.render_root.use_inner_render_root_ref(|rr| {
            let Ok(mut write) = self.window_event_handler.try_borrow_mut() else {
                return;
            };
            write.cleanup(&rr.tree);
            write.shrink_to_fit();
        });
    }
    pub(crate) fn new<V>(args: WindowNew<'_, V, W>) -> Result<Self, crate::error::Error>
    where
        V: FnOnce() -> NewWidget<dyn Widget>,
    {
        let window_handle = args.handle();
        let WindowNew {
            window,
            view,
            default_properties,
            access_kit,
            app_handle,
            signal_sender,
            parent_owner,
            base_color,
            factory,
        } = args;
        let window_owner = parent_owner.child();
        let event_handlers = InternWindowEventHandler::default();

        let size = window.inner_size();

        let renderer = factory.create(&app_handle);

        let render_root = InnerRenderRoot::new(
            {
                let window = window.id();
                let proxy = app_handle.get_proxy().clone();
                move |ev| {
                    let _ = signal_sender.send((window, ev));
                    let _ = proxy.send_event(crate::app::EventLoopEvent::HandleRenderRootSignals);
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
                provide_context(window_handle.clone());
                provide_context(app_handle);
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

        let this = Self {
            renderer,
            winit_window: window,
            render_root,
            owner: window_owner,
            access_kit,
            event_reducer: WindowEventReducer::default(),
            last_anim: None,
            window_event_handler: event_handlers,
            base_color: base_color.unwrap_or(BLACK),
            handle: window_handle,
        };
        Ok(this)
    }
    fn sync_surface_render_root_size(&mut self) -> bool {
        let Some(size) = self
            .render_root
            .use_inner_render_root_ref(|root| root.tree.size())
        else {
            return false;
        };
        self.renderer.set_size(size.width, size.height);
        true
    }
    pub fn render(&mut self) -> Result<(), crate::error::Error> {
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

        let Some(((visual_plan, _access_tree), size)) = self
            .render_root
            .use_inner_render_root_mut(|root| (root.tree.redraw(), root.tree.size()))
        else {
            return Ok(());
        };

        let overlays: Vec<_> = visual_plan
            .overlay_layers()
            .map(|layer| {
                let VisualLayerKind::Scene(scene) = &layer.kind else {
                    unreachable!("overlay_layers only returns scene layers");
                };
                ImagingLayer {
                    scene,
                    transform: layer.transform,
                }
            })
            .collect();
        let root_layer = visual_plan
            .root_layer()
            .expect("paint should always produce a root layer");
        let VisualLayerKind::Scene(root_scene) = &root_layer.kind else {
            unreachable!("root_layer always returns a scene layer");
        };
        let mut frame = PreparedFrame::new(
            size.width,
            size.height,
            self.winit_window.scale_factor(),
            self.base_color,
            root_scene,
            &overlays,
        );

        if self.sync_surface_render_root_size() {
            self.renderer.render(|painter| {
                frame.paint_into(painter);
            });
        }
        if let Some(access_tree) = _access_tree {
            self.access_kit.update_if_active(|| access_tree);
        }
        Ok(())
    }
    pub fn create_children_owner(&self) -> Owner {
        self.owner.child()
    }
    pub fn get_handle(&self) -> WindowHandle {
        self.handle.clone()
    }
    pub fn resume<F: FnOnce() + 'static>(&mut self, on_ready: F) {
        let Some(size) = self.render_root.use_render_root_ref(|root| root.size()) else {
            return;
        };
        self.renderer
            .resume(self.winit_window.clone(), size.width, size.height, on_ready);
    }
    pub fn suspend(&mut self) {
        self.renderer.suspend();
    }
    pub fn complete_resume(&mut self) -> bool {
        self.renderer.complete_resume()
    }
}
