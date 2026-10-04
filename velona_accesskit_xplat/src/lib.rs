use std::sync::Arc;

use accesskit::{Rect, TreeUpdate};
use velona_core_accesskit::{Adapter, AdapterFactory};
use winit::window::Window;

pub struct XPlatAdapter {
    adapter: accesskit_xplat::Adapter,
}

impl Adapter for XPlatAdapter {
    fn handle_winit_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        window: &dyn Window,
    ) {
        match event {
            winit::event::WindowEvent::Focused(is_focused) => {
                self.adapter.set_focus(*is_focused);
            }
            winit::event::WindowEvent::Moved(_) | winit::event::WindowEvent::SurfaceResized(_) => {
                let outer_position: (_, _) = window
                    .outer_position()
                    .unwrap_or_default()
                    .cast::<f64>()
                    .into();
                let outer_size: (_, _) = window.outer_size().cast::<f64>().into();
                let inner_position: (_, _) = window.surface_position().cast::<f64>().into();
                let inner_size: (_, _) = window.surface_size().cast::<f64>().into();

                self.adapter.set_window_bounds(
                    Rect::from_origin_size(outer_position, outer_size),
                    Rect::from_origin_size(inner_position, inner_size),
                )
            }
            _ => {}
        }
    }

    fn update_if_active(&mut self, updater: Box<dyn FnOnce() -> TreeUpdate>) {
        self.adapter.update_if_active(updater);
    }
}

pub struct XPlatAdapterFactory;

struct XPlatEventHandler {
    handler: Box<dyn velona_core_accesskit::EventHandler>,
}

impl accesskit_xplat::EventHandler for XPlatEventHandler {
    fn handle_accesskit_event(&self, event: accesskit_xplat::WindowEvent) {
        self.handler.handle_accesskit_event(match event {
            accesskit_xplat::WindowEvent::InitialTreeRequested => {
                velona_core_accesskit::WindowEvent::InitialTreeRequested
            }
            accesskit_xplat::WindowEvent::ActionRequested(action_request) => {
                velona_core_accesskit::WindowEvent::ActionRequested(action_request)
            }
            accesskit_xplat::WindowEvent::AccessibilityDeactivated => {
                velona_core_accesskit::WindowEvent::AccessibilityDeactivated
            }
        });
    }
}

impl AdapterFactory for XPlatAdapterFactory {
    type Adapter
        = XPlatAdapter
    where
        Self: Sized;
    fn create_adapter(
        &mut self,
        args: velona_core_accesskit::CreateAdapterArgs<'_>,
    ) -> Option<Self::Adapter>
    where
        Self: Sized,
    {
        let handle = {
            #[cfg(target_os = "android")]
            {
                use winit::platform::android::ActiveEventLoopExtAndroid;

                args.active_event_loop.android_app()
            }
            #[cfg(not(target_os = "android"))]
            {
                use raw_window_handle::HasWindowHandle;

                args.window.window_handle().ok()?.as_raw()
            }
        };
        let adapter = accesskit_xplat::Adapter::with_combined_handler(
            handle,
            Arc::new(XPlatEventHandler {
                handler: args.event_handler,
            }),
        );
        Some(XPlatAdapter { adapter })
    }

    fn create_erased_adapter(
        &mut self,
        args: velona_core_accesskit::CreateAdapterArgs<'_>,
    ) -> Option<Box<dyn Adapter>> {
        let a = self.create_adapter(args)?;
        Some(Box::new(a))
    }
}
