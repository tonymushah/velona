use std::marker::PhantomData;

use masonry::core::{Widget, WidgetId};

#[derive(Debug, Clone)]
pub struct VelonaWidgetRef<W>
where
    W: Widget + 'static,
{
    id: WidgetId,
    phantom: PhantomData<W>,
}
