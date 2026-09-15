use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use masonry_raw_box::RawBox;
use reactive_stores::{ArcStore, Patch, Store};
use velona_core::{
    AnyNewWidget, NewWidgetExt,
    masonry_core::core::Widget,
    reactive::{
        effect::Effect,
        owner::{provide_context, use_context},
        traits::{Get, GetUntracked, Read},
    },
    utils::ConsumeResult,
};

pub type ViewFn = Box<dyn Fn() -> AnyNewWidget + Send>;

#[derive(Debug, Store, Patch)]
pub(crate) struct RouterContextInner {
    current_route: String,
}

const DEFAULT_CURRENT_ROUTE: &str = "/";

impl Default for RouterContextInner {
    fn default() -> Self {
        Self {
            current_route: DEFAULT_CURRENT_ROUTE.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouterContext(ArcStore<RouterContextInner>);

impl RouterContext {
    pub fn goto<R>(&self, route: R)
    where
        R: Into<String>,
    {
        self.0.clone().current_route().patch(route.into());
    }

    pub fn current_route_untracked(&self) -> String {
        self.0.clone().current_route().get_untracked()
    }

    pub fn current_route(&self) -> String {
        self.0.clone().current_route().get()
    }
}

#[derive(derive_more::Debug, Default)]
pub struct RouterBuilder {
    #[debug("{:#?}", routes.keys().collect::<Box<[_]>>())]
    routes: HashMap<String, ViewFn>,
    starting_route: Option<String>,
    #[debug(ignore)]
    fallback: Option<ViewFn>,
}

pub fn except_router_ctx() -> RouterContext {
    use_context().expect("Cannot get the Router Context")
}

impl RouterBuilder {
    pub fn route<R, V>(mut self, route: R, view: V) -> Self
    where
        R: Into<String>,
        V: Fn() -> AnyNewWidget + Send + 'static,
    {
        let _ = self.routes.insert(route.into(), Box::new(view));
        self
    }

    pub fn starting_route<R>(mut self, route: R) -> Self
    where
        R: Into<String>,
    {
        self.starting_route = Some(route.into());
        self
    }

    pub fn unset_starting_route(mut self) -> Self {
        let _ = self.starting_route.take();
        self
    }

    pub fn fallback<V>(mut self, view: V) -> Self
    where
        V: Fn() -> AnyNewWidget + Send + 'static,
    {
        self.fallback = Some(Box::new(view));
        self
    }

    pub fn build(self) -> impl FnOnce() -> AnyNewWidget + Send + 'static {
        let router_ctx = RouterContextInner {
            current_route: self.starting_route.unwrap_or(DEFAULT_CURRENT_ROUTE.into()),
        };
        let router_ctx = ArcStore::new(router_ctx);

        let routes = Arc::new(Mutex::new(self.routes));

        let fallback = Arc::new(Mutex::new(self.fallback));

        move || {
            let box_ = RawBox::empty().prepare();

            let _ref = box_.create_velona_ref();

            provide_context(RouterContext(router_ctx.clone()));

            Effect::new(move || {
                routes.clear_poison();

                let current_route = &*router_ctx.clone().current_route().read();

                let widget = match routes.lock().unwrap().get(current_route).as_ref() {
                    Some(route) => route(),
                    None => {
                        let fallback_lock = fallback.lock().unwrap();
                        let Some(fallback) = fallback_lock.as_ref() else {
                            _ref.edit_local_now(|mut this| {
                                RawBox::remove_child(&mut this);
                            })
                            .consume_with_log_err();
                            return;
                        };
                        fallback()
                    }
                };
                _ref.edit_local_now(|mut this| {
                    RawBox::set_child(&mut this, widget);
                })
                .consume_with_log_err();
            });

            box_.erased()
        }
    }
}
