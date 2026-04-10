use masonry::{
    core::{ArcStr, NewWidget, StyleProperty, Widget},
    widgets::Label,
};
use reactive_graph::graph::untrack;

use super::NewWidgetExt;

pub trait NewLabelExt {
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>;
    fn style<S, T>(self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>;
}

impl NewLabelExt for NewWidget<Label> {
    fn text<S, T>(self, text: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<ArcStr>,
    {
        self.use_reactive_widget_mut(move |mut this| {
            Label::set_text(&mut this, text());
        })
    }

    fn style<S, T>(mut self, style: S) -> Self
    where
        S: Fn() -> T + 'static,
        T: Into<StyleProperty>,
    {
        self.widget = Box::new(self.widget.with_style(untrack(&style)));
        self.use_reactive_widget_mut(move |mut this| {
            Label::insert_style(&mut this, style());
        })
    }
}

pub fn label<S, T>(text: S) -> NewWidget<Label>
where
    S: Fn() -> T + 'static,
    T: Into<ArcStr>,
{
    Label::new(untrack(&text)).with_auto_id().text(text)
}
