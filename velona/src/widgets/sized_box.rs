use masonry::{
    core::{NewWidget, Widget},
    layout::Length,
    widgets::SizedBox,
};
use reactive_graph::effect::Effect;

use crate::{
    AnyNewWidget, NewWidgetExt,
    widgets::{ReactiveSingleChildExt, SingleChildWidget},
};

pub trait NewSizedBoxExt {
    /// Set a "reactive" child for this [`SizedBox`].
    ///
    /// The `child_fn` will run inside a [`Effect::new`].
    ///
    /// If the function returns a [`NewWidget`], it will [update](SizedBox::set_child) the current child,
    /// if [`None`], the current child will be [removed](SizedBox::remove_child).
    fn child_opt<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> Option<AnyNewWidget> + 'static;
    /// Set a reactive width for this [`SizedBox`].
    ///
    /// The `width_fn` will run inside a [`Effect::new`],
    ///
    /// If the function returns a [`Length`], it will [update the current width](SizedBox::set_width) sized box,
    /// if [`None`], the current container width will be [unset](SizedBox::unset_width).
    fn raw_width<W>(self, width_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static;
    /// Similar to [`width_opt`](Self::raw_width)
    fn width<W>(self, width_fn: W) -> Self
    where
        W: Fn() -> Length + 'static,
        Self: Sized,
    {
        self.raw_width(move || Some(width_fn()))
    }
    /// Set a reactive height for this [`SizedBox`].
    ///
    /// The `height_fn` will run inside a [`Effect::new`],
    ///
    /// If the function returns a [`Length`], it will [update the current height](SizedBox::set_height) sized box,
    /// if [`None`], the current container height will be [unset](SizedBox::unset_height).
    fn raw_height<W>(self, height_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static;
    /// Similar to [`height_opt`](Self::raw_height)
    fn height<W>(self, height_fn: W) -> Self
    where
        W: Fn() -> Length + 'static,
        Self: Sized,
    {
        self.raw_height(move || Some(height_fn()))
    }
}

impl NewSizedBoxExt for NewWidget<SizedBox> {
    fn child_opt<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> Option<AnyNewWidget> + 'static,
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

    fn raw_width<W>(self, width_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static,
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

    fn raw_height<W>(self, height_fn: W) -> Self
    where
        W: Fn() -> Option<Length> + 'static,
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
}

impl SingleChildWidget for NewWidget<SizedBox> {
    fn use_child_erased<C>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(masonry::core::WidgetMut<'_, dyn Widget>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            if let Some(child) = SizedBox::child_mut(&mut this) {
                use_child_fn(child);
            } else {
                log::warn!("Not child for SizedBox #{}", this.id());
            }
        })
    }
}

impl ReactiveSingleChildExt for NewWidget<SizedBox> {
    fn child<Cf>(self, child_fn: Cf) -> Self
    where
        Cf: Fn() -> AnyNewWidget + 'static,
    {
        self.child_opt(move || Some(child_fn()))
    }
}
