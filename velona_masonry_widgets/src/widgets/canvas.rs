//! Various [`Canvas`] implementations.
//!
// As the [`masonry::widgets::Canvas`] documentation says:
// > A widget allowing custom drawing.
// > A canvas takes a [painter callback](Canvas::update_scene);
// > every time the canvas is repainted,
// > that callback is run with an [`imaging::record::Scene`].
// > That recording is then replayed as the canvas contents.
//! In `velona`, there are 2 ways to paint a [`Canvas`]:
//!
//! 1. [`NewCanvasExt::update_scene`]:
//!
//! > **Pros**:
//! > - Inline with [`NewWidget<Canvas>`]
//! > - Runs inside an [`Effect`]
//!
//! > **Cons**:
//! > - Might be complex to use (You might find yourself using a bunch of signals for basic stuff).
//!
//! 2. [`CanvasRefExt::update_scene`]:
//!
//! > **Pros**:
//! > - Can be called on another thread.
//! > - Should always succeed in component initialization.
//!
//! > **Cons**:
//! > - Fails if the canvas is not in the widget tree (ex: the widget has been destroyed).
//! > - Requires the callback function to be [`Send`] and [`Sync`].
//!
//! I personally recommend using the [`CanvasRefExt::update_scene`] function.
//!
//! _See the [widget](Canvas) documentation for more information_.

use std::fmt::Debug;

use masonry::imaging::record::Scene;
use masonry::kurbo::Size;
use masonry::{
    core::{ArcStr, MutateCtx, NewWidget},
    widgets::Canvas,
};
use velona_core::widget_ref::{UseWidgetFromRefError, VelonaWidgetRef};
use velona_core::widgets::UseWidgetValResult;

use crate::NewWidgetExt;

#[non_exhaustive]
pub struct UpdateSceneCtx<'a, 'b> {
    pub mutate: &'a mut MutateCtx<'b>,
    pub scene: &'a mut Scene,
    pub size: Size,
}

impl<'a, 'b> Debug for UpdateSceneCtx<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateSceneCtx")
            .field("scene", &self.scene)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

/// A [new](NewWidget) [`Canvas`] trait extension.
// TODO add drawing example
#[must_use]
pub trait NewCanvasExt {
    /// Updates the canvas scene.
    ///
    /// It is worth noting that this function run inside an [`Effect`].
    ///
    /// _I personally don't recommend using this for updating the scene of a canvas,
    /// i recommend using a [`VelonaWidgetRef`] since it give you more "freedom" (aka thread-safety) on what to show._
    fn update_scene<Vfn, Ufn, V, O>(self, val_fn: Vfn, update_fn: Ufn) -> Self
    where
        Ufn: FnMut(UpdateSceneCtx<'_, '_>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;

    /// Sets the text that will describe the canvas to screen readers.
    ///
    /// See [`Canvas::with_alt_text`] for details.
    fn alt_text<T>(self, alt_text: T) -> Self
    where
        T: Fn() -> Option<ArcStr> + 'static;
}

impl NewCanvasExt for NewWidget<Canvas> {
    fn update_scene<Vfn, Ufn, V, O>(self, val_fn: Vfn, mut update_fn: Ufn) -> Self
    where
        Ufn: FnMut(UpdateSceneCtx<'_, '_>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            Canvas::update_scene(&mut this, |ctx, scene, size| {
                let _ctx = UpdateSceneCtx {
                    scene,
                    mutate: ctx,
                    size,
                };
                update_fn(_ctx, val);
            });
        })
    }
    fn alt_text<T>(self, alt_text: T) -> Self
    where
        T: Fn() -> Option<ArcStr> + 'static,
    {
        self.use_widget_mut(alt_text, |mut this, alt_text| {
            Canvas::set_alt_text(&mut this, alt_text);
        })
    }
}

/// A [`Canvas`] [ref](VelonaWidgetRef) trait extension.
pub trait CanvasRefExt {
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
    fn update_scene<U>(self, updates: U) -> Result<(), UseWidgetFromRefError>
    where
        U: FnOnce(&mut MutateCtx<'_>, &mut Scene, Size) + Send + Sync + 'static,
    {
        self.edit(move |mut this| {
            Canvas::update_scene(&mut this, updates);
        })
    }
}
