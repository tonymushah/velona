// TODO Add example
use masonry::{
    core::NewWidget,
    widgets::{Label, SelectorItem},
};

use crate::{NewWidgetExt, widgets::TypedSingleChildWidget};

impl TypedSingleChildWidget for NewWidget<SelectorItem> {
    type Child = Label;

    fn use_child<C>(self, mut use_child_fn: C) -> Self
    where
        C: FnMut(masonry::core::WidgetMut<'_, Self::Child>) + 'static,
    {
        self.use_reactive_widget_mut(move |mut this| {
            use_child_fn(SelectorItem::child_mut(&mut this));
        })
    }
}
