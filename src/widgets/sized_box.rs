use masonry::{core::NewWidget, properties::types::Length, widgets::SizedBox};
use reactive_graph::effect::Effect;

use crate::{AnyNewWidget, NewWidgetExt};

pub trait NewSizedBoxExt {
    /// Set a "reactive" child for this [`SizedBox`].
    ///
    /// The `child_fn` will run inside a [`Effect::new`].
    ///
    /// If the function returns a [`NewWidget`], it will [update](SizedBox::set_child) the current child,
    /// if [`None`], the current child will be [removed](SizedBox::remove_child).
    fn child_opt<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: FnMut() -> Option<AnyNewWidget> + 'static;
    /// Similar to [`child`](Self::child_opt).
    fn child<Cf>(self, mut child_fn: Cf) -> Self
    where
        Cf: FnMut() -> AnyNewWidget + 'static,
        Self: Sized,
    {
        self.child_opt(move || Some(child_fn()))
    }
    /// Set a reactive width for this [`SizedBox`].
    ///
    /// The `width_fn` will run inside a [`Effect::new`],
    ///
    /// If the function returns a [`Length`], it will [update the current width](SizedBox::set_width) sized box,
    /// if [`None`], the current container width will be [unset](SizedBox::unset_width).
    fn width_opt<W>(self, width_fn: W) -> Self
    where
        W: FnMut() -> Option<Length> + 'static;
    /// Similar to [`width_opt`](Self::width_opt)
    fn width<W>(self, mut width_fn: W) -> Self
    where
        W: FnMut() -> Length + 'static,
        Self: Sized,
    {
        self.width_opt(move || Some(width_fn()))
    }
    /// Set a reactive height for this [`SizedBox`].
    ///
    /// The `height_fn` will run inside a [`Effect::new`],
    ///
    /// If the function returns a [`Length`], it will [update the current height](SizedBox::set_height) sized box,
    /// if [`None`], the current container height will be [unset](SizedBox::unset_height).
    fn height_opt<W>(self, height_fn: W) -> Self
    where
        W: FnMut() -> Option<Length> + 'static;
    /// Similar to [`height_opt`](Self::height_opt)
    fn height<W>(self, mut height_fn: W) -> Self
    where
        W: FnMut() -> Length + 'static,
        Self: Sized,
    {
        self.height_opt(move || Some(height_fn()))
    }
    /// Set a reactive raw_width for this [`SizedBox`].
    ///
    /// The `width_fn` will run inside a [`Effect::new`] and the return value will [update the current `raw_width`](SizedBox::set_raw_width) sized box,
    fn raw_width<W>(self, width_fn: W) -> Self
    where
        W: FnMut() -> Option<f64> + 'static;
    /// Set a reactive raw_height for this [`SizedBox`].
    ///
    /// The `height_fn` will run inside a [`Effect::new`] and the return value will [update the current `raw_height`](SizedBox::set_raw_height) sized box,
    fn raw_height<W>(self, height_fn: W) -> Self
    where
        W: FnMut() -> Option<f64> + 'static;
}

impl NewSizedBoxExt for NewWidget<SizedBox> {
    fn child_opt<Cf>(self, mut child_fn: Cf) -> Self
    where
        Cf: FnMut() -> Option<AnyNewWidget> + 'static,
    {
        let w_ref = self.create_velona_ref();
        Effect::new(move || {
            let maybe_new_widget = child_fn();
            if let Some(new_widget) = maybe_new_widget {
                let _ = w_ref
                    .edit_local_now(|mut this| {
                        SizedBox::set_child(&mut this, new_widget);
                    })
                    .inspect_err(|err| {
                        log::error!("Cannot set a new child for this sized box => {err}");
                    });
            } else {
                let _ = w_ref
                    .edit_local_now(|mut this| {
                        SizedBox::remove_child(&mut this);
                    })
                    .inspect_err(|err| {
                        log::error!("Cannot remove child for this sized box => {err}");
                    });
            }
        });
        self
    }

    fn width_opt<W>(self, mut width_fn: W) -> Self
    where
        W: FnMut() -> Option<Length> + 'static,
    {
        let w_ref = self.create_velona_ref();
        Effect::new(move || {
            let maybe_new_width = width_fn();
            if let Some(new_width) = maybe_new_width {
                let _ = w_ref
                    .edit_local_now(|mut this| {
                        SizedBox::set_width(&mut this, new_width);
                    })
                    .inspect_err(|err| {
                        log::error!("Cannot set a new width for sized box => {err}");
                    });
            } else {
                let _ = w_ref
                    .edit_local_now(|mut this| {
                        SizedBox::unset_width(&mut this);
                    })
                    .inspect_err(|err| {
                        log::error!("Cannot unset width for this sized box => {err}");
                    });
            }
        });
        self
    }

    fn height_opt<W>(self, mut height_fn: W) -> Self
    where
        W: FnMut() -> Option<Length> + 'static,
    {
        let w_ref = self.create_velona_ref();
        Effect::new(move || {
            let maybe_new_height = height_fn();
            if let Some(new_height) = maybe_new_height {
                let _ = w_ref
                    .edit_local_now(|mut this| {
                        SizedBox::set_height(&mut this, new_height);
                    })
                    .inspect_err(|err| {
                        log::error!("Cannot set a new height for sized box => {err}");
                    });
            } else {
                let _ = w_ref
                    .edit_local_now(|mut this| {
                        SizedBox::unset_height(&mut this);
                    })
                    .inspect_err(|err| {
                        log::error!("Cannot unset height for this sized box => {err}");
                    });
            }
        });
        self
    }

    fn raw_width<W>(self, mut width_fn: W) -> Self
    where
        W: FnMut() -> Option<f64> + 'static,
    {
        let w_ref = self.create_velona_ref();
        Effect::new(move || {
            let new_raw_width = width_fn();
            let _ = w_ref
                .edit_local_now(|mut this| {
                    SizedBox::set_raw_width(&mut this, new_raw_width);
                })
                .inspect_err(|err| {
                    log::error!("Cannot set a new raw_width for sized box => {err}");
                });
        });
        self
    }

    fn raw_height<W>(self, mut height_fn: W) -> Self
    where
        W: FnMut() -> Option<f64> + 'static,
    {
        let w_ref = self.create_velona_ref();
        Effect::new(move || {
            let new_raw_height = height_fn();
            let _ = w_ref
                .edit_local_now(|mut this| {
                    SizedBox::set_raw_height(&mut this, new_raw_height);
                })
                .inspect_err(|err| {
                    log::error!("Cannot set a new raw_height for sized box => {err}");
                });
        });
        self
    }
}
