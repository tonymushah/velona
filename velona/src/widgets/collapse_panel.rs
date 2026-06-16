use masonry::{
    core::{ArcStr, NewWidget, WidgetMut},
    widgets::{CollapsePanel, DisclosureButton, Label},
};

use crate::NewWidgetExt;

#[cfg(doc)]
use super::{ReactiveSingleChildExt, SingleChildWidget};
#[cfg(doc)]
use reactive_graph::effect::Effect;

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
    /// It is worth noting that this function will run inside an [`Effect`] _which means that it will re-run on signal changes_.
    fn use_disclosure_button<F>(self, use_fn: F) -> Self
    where
        F: FnMut(WidgetMut<'_, DisclosureButton>) + 'static;
    /// Use the [header label](CollapsePanel::header_label_mut).
    ///
    /// It is worth noting that this function will run inside an [`Effect`].
    fn use_header_label<F>(self, use_fn: F) -> Self
    where
        F: FnMut(WidgetMut<'_, Label>) + 'static;
}

impl NewCollapsePanelExt for NewWidget<CollapsePanel> {
    fn collapsed<C>(self, collapsed: C) -> Self
    where
        C: Fn() -> bool + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            CollapsePanel::set_collapsed(&mut this, collapsed());
        })
    }

    fn text<Tf, T>(self, text: Tf) -> Self
    where
        Tf: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_reactive_widget_mut(move |mut this| {
            CollapsePanel::set_text(&mut this, text().into());
        })
    }

    fn use_disclosure_button<F>(self, mut use_fn: F) -> Self
    where
        F: FnMut(WidgetMut<'_, DisclosureButton>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(CollapsePanel::disclosure_button_mut(&mut this));
        })
    }

    fn use_header_label<F>(self, mut use_fn: F) -> Self
    where
        F: FnMut(WidgetMut<'_, Label>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_fn(CollapsePanel::header_label_mut(&mut this))
        })
    }
}
