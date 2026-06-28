use imaging::record::Scene;
use masonry::kurbo::Size;
use masonry::{
    core::{ArcStr, MutateCtx, NewWidget},
    widgets::Canvas,
};
use reactive_graph::effect::Effect;

use crate::widget_ref::{EditWidgetLocalError, UseWidgetFromRefError, VelonaWidgetRef};
use crate::{NewWidgetExt, utils::ConsumeResult};

/// A [new](NewWidget) [`Canvas`] trait extension.
// TODO add drawing example
pub trait NewCanvasExt {
    /// Updates the canvas scene.
    ///
    /// It is worth noting that this function run inside an [`Effect`].
    ///
    /// _I personally don't recommend using this for updating the scene of a canvas,
    /// i recommend using a [`VelonaWidgetRef`] since it give you more "freedom" (aka thread-safety) on what to show._
    fn update_scene<U>(self, updates: U) -> Self
    where
        U: FnMut(&mut MutateCtx<'_>, &mut Scene, Size) + 'static;
    /// Sets the text that will describe the canvas to screen readers.
    ///
    /// See [`Canvas::with_alt_text`] for details.
    fn alt_text<T>(self, alt_text: T) -> Self
    where
        T: Fn() -> Option<ArcStr> + 'static;
}

impl NewCanvasExt for NewWidget<Canvas> {
    fn update_scene<U>(self, mut updates: U) -> Self
    where
        U: FnMut(&mut MutateCtx<'_>, &mut Scene, Size) + 'static,
    {
        let c_ref = self.create_velona_ref();
        Effect::new(move || {
            c_ref
                .edit_local_now(|mut this| {
                    Canvas::update_scene(&mut this, |ctx, sc, size| {
                        updates(ctx, sc, size);
                    });
                })
                .consume_with_log_err();
        });
        self
    }

    fn alt_text<T>(self, alt_text: T) -> Self
    where
        T: Fn() -> Option<ArcStr> + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Canvas::set_alt_text(&mut this, alt_text());
        })
    }
}

/// A [`Canvas`] [ref](VelonaWidgetRef) trait extension.
pub trait CanvasRefExt {
    /// Updates the canvas scene.
    ///
    /// It is worth noting that this function doesn't run inside an [`Effect`].
    ///
    /// *See [`VelonaWidgetRef::edit_local_now`] for more details*.
    fn update_scene_local<U>(self, updates: U) -> Result<(), EditWidgetLocalError>
    where
        U: FnOnce(&mut MutateCtx<'_>, &mut Scene, Size) + 'static;
    /// Updates the canvas scene.
    ///
    /// It is worth noting that this function doesn't run inside an [`Effect`].
    ///
    /// *See [`VelonaWidgetRef::edit`] for more details*.
    fn update_scene<U>(self, updates: U) -> Result<(), UseWidgetFromRefError>
    where
        U: FnOnce(&mut MutateCtx<'_>, &mut Scene, Size) + Send + Sync + 'static;
}

impl CanvasRefExt for VelonaWidgetRef<Canvas> {
    fn update_scene_local<U>(self, updates: U) -> Result<(), EditWidgetLocalError>
    where
        U: FnOnce(&mut MutateCtx<'_>, &mut Scene, Size) + 'static,
    {
        self.edit_local_now(|mut this| {
            Canvas::update_scene(&mut this, updates);
        })
    }

    fn update_scene<U>(self, updates: U) -> Result<(), UseWidgetFromRefError>
    where
        U: FnOnce(&mut MutateCtx<'_>, &mut Scene, Size) + Send + Sync + 'static,
    {
        self.edit(move |mut this| {
            Canvas::update_scene(&mut this, updates);
        })
    }
}
