//! Various [`CollapsePanel`] implementations.
//!
//! The most important thing in the module is the [`NewCollapsePanelExt`]
//! which is implemented for [`NewWidget<CollapsePanel>`].
//!
//! The [`NewWidget<CollapsePanel>`] also implements the [`ReactiveSingleChildExt`] trait
//! and the [`SingleChildWidget`] trait.
//!
//! _See the [widget](CollapsePanel) documentation for more information_.

use masonry::{
    core::{ArcStr, NewWidget, WidgetMut},
    widgets::{CollapsePanel, DisclosureButton, Label},
};
use velona_core::widgets::{UseWidgetValResult, View};

use crate::NewWidgetExt;

#[cfg(doc)]
use super::{ReactiveSingleChildExt, SingleChildWidget};
#[cfg(doc)]
use velona_core::reactive::effect::Effect;

/// A [`CollapsePanel`] extension trait.
///
/// If you want to change the child reactively, use [`ReactiveSingleChildExt::child`].
///
/// It you want to use the child, use [`SingleChildWidget`].
// TODO add an example for this
pub trait NewCollapsePanelExt {
    /// Set the [collapsed](CollapsePanel::set_collapsed) value reactively.
    fn collapsed<C>(self, collapsed: C) -> Self
    where
        C: Fn() -> bool + 'static;
    /// Set the [text](CollapsePanel::set_text) reactively.
    fn text<Tf, T>(self, text: Tf) -> Self
    where
        Tf: Fn() -> T + 'static,
        T: Into<ArcStr>;
    /// Use the [discolure button](CollapsePanel::disclosure_button_mut).
    ///
    /// It is worth noting that only the `val_fn` function will run inside an [`Effect`] _which means that it will re-run on signal changes_.
    fn use_disclosure_button<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, DisclosureButton>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
    /// Use the [header label](CollapsePanel::header_label_mut).
    ///
    /// It is worth noting that only the `val_fn` function will run inside an [`Effect`].
    fn use_header_label<Vfn, Efn, V, O>(self, val_fn: Vfn, edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, Label>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static;
}

impl NewCollapsePanelExt for NewWidget<CollapsePanel> {
    fn collapsed<C>(self, collapsed: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_widget_mut(collapsed, |mut this, collapsed| {
            CollapsePanel::set_collapsed(&mut this, collapsed);
        })
    }

    fn text<Tf, T>(self, text: Tf) -> Self
    where
        Tf: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_widget_mut(
            move || text().into(),
            |mut this, text| {
                CollapsePanel::set_text(&mut this, text);
            },
        )
    }

    fn use_disclosure_button<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, DisclosureButton>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(CollapsePanel::disclosure_button_mut(&mut this), val);
        })
    }

    fn use_header_label<Vfn, Efn, V, O>(self, val_fn: Vfn, mut edit_fn: Efn) -> Self
    where
        Efn: FnMut(WidgetMut<'_, Label>, V) + 'static,
        V: 'static,
        O: 'static,
        Vfn: Fn(Option<O>) -> UseWidgetValResult<V, O> + 'static,
    {
        self.use_widget_mut_val(val_fn, move |mut this, val| {
            edit_fn(CollapsePanel::header_label_mut(&mut this), val);
        })
    }
}

pub trait IntoCollapsePanel {
    fn into_collapse_panel<T>(self, collpse: bool, header_text: T) -> CollapsePanel
    where
        T: Into<ArcStr>;
}

impl<V> IntoCollapsePanel for V
where
    V: View + 'static,
{
    fn into_collapse_panel<T>(self, collpse: bool, header_text: T) -> CollapsePanel
    where
        T: Into<ArcStr>,
    {
        CollapsePanel::new(collpse, header_text, self.into_new_widget())
    }
}
