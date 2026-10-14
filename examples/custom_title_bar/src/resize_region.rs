use masonry_widget_wrappers::WidgetWrapper;
use tracing::trace_span;
use velona::masonry::{
    core::{CursorIcon, FromDynWidget, ResizeDirection, Widget},
    kurbo::{Point, Rect},
};

pub struct DragRegions {
    north_west: Rect,
    north: Rect,
    north_east: Rect,
    east: Rect,
    south_east: Rect,
    south: Rect,
    south_west: Rect,
    west: Rect,
}

impl DragRegions {
    fn get_resize_region(&self, point: Point) -> Option<ResizeDirection> {
        if self.north_west.contains(point) {
            Some(ResizeDirection::NorthWest)
        } else if self.north.contains(point) {
            Some(ResizeDirection::North)
        } else if self.north_east.contains(point) {
            Some(ResizeDirection::NorthEast)
        } else if self.east.contains(point) {
            Some(ResizeDirection::East)
        } else if self.south_east.contains(point) {
            Some(ResizeDirection::SouthEast)
        } else if self.south.contains(point) {
            Some(ResizeDirection::South)
        } else if self.south_west.contains(point) {
            Some(ResizeDirection::SouthWest)
        } else if self.west.contains(point) {
            Some(ResizeDirection::West)
        } else {
            None
        }
    }
}

pub fn get_drag_regions(outer: Rect, inner: Rect) -> Option<DragRegions> {
    if outer.contains_rect(inner) {
        let north_west = Rect::new(outer.x0, outer.y0, inner.x0, inner.y0);

        let north = Rect::new(inner.x0, outer.y0, inner.x1, inner.y0);

        let north_east = Rect::new(inner.x1, outer.y0, outer.x1, inner.y0);

        let east = Rect::new(inner.x1, inner.y0, outer.x1, inner.y1);

        let south_east = Rect::new(inner.x1, inner.y1, outer.x1, outer.y1);

        let south = Rect::new(inner.x0, inner.y1, inner.x1, outer.y1);

        let south_west = Rect::new(outer.x0, inner.y1, inner.x0, outer.y1);

        let west = Rect::new(outer.x0, inner.y0, inner.x0, inner.y1);

        Some(DragRegions {
            north_west,
            north,
            north_east,
            east,
            south_east,
            south,
            south_west,
            west,
        })
    } else {
        None
    }
}

pub struct ResizeRegion<W: Widget + FromDynWidget + ?Sized> {
    inner: W,
}

impl<W: Widget + 'static> ResizeRegion<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W> WidgetWrapper for ResizeRegion<W>
where
    W: Widget + FromDynWidget,
{
    type Action
        = W::Action
    where
        Self: Sized;

    fn inner(&self) -> &dyn velona::masonry::core::Widget {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut dyn velona::masonry::core::Widget {
        &mut self.inner
    }

    fn make_trace_span(&self, id: velona::masonry::core::WidgetId) -> tracing::Span {
        trace_span!("ResizeRegion", id = id.trace())
    }
    fn on_pointer_event(
        &mut self,
        ctx: &mut velona::masonry::core::EventCtx<'_>,
        _props: &mut velona::masonry::core::PropertiesMut<'_>,
        event: &velona::masonry::core::PointerEvent,
    ) {
        if let velona::masonry::core::PointerEvent::Down(pointer_button_event) = event
            && pointer_button_event.pointer.is_primary_pointer()
            && let Some(drag_region) = get_drag_regions(ctx.border_box(), ctx.content_box())
            && let Some(resize_region) = drag_region
                .get_resize_region(ctx.local_position(pointer_button_event.state.position))
        {
            ctx.drag_resize_window(resize_region);
        }
    }
    fn get_cursor(
        &self,
        ctx: &velona::masonry::core::QueryCtx<'_>,
        pos: Point,
    ) -> velona::masonry::core::CursorIcon {
        if let Some(drag_region) = get_drag_regions(ctx.border_box(), ctx.content_box())
            && let Some(resize_region) = drag_region.get_resize_region(ctx.to_local(pos))
        {
            match resize_region {
                ResizeDirection::East => CursorIcon::EResize,
                ResizeDirection::North => CursorIcon::NResize,
                ResizeDirection::NorthEast => CursorIcon::NeResize,
                ResizeDirection::NorthWest => CursorIcon::NwResize,
                ResizeDirection::South => CursorIcon::SResize,
                ResizeDirection::SouthEast => CursorIcon::SeResize,
                ResizeDirection::SouthWest => CursorIcon::SwResize,
                ResizeDirection::West => CursorIcon::WResize,
            }
        } else {
            self.inner.get_cursor(ctx, pos)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_drag_region() {
        let a_rect = Rect::new(0., 0., 10., 12.);
        let b_rect = Rect::new(1., 2., 9., 10.);

        let drag_regions = get_drag_regions(a_rect, b_rect).expect("There should be a drag region");

        assert_eq!(drag_regions.north_west, Rect::new(0., 0., 1., 2.));

        assert_eq!(drag_regions.north, Rect::new(1., 0., 9., 2.));

        assert_eq!(drag_regions.north_east, Rect::new(9., 0., 10., 2.));

        assert_eq!(drag_regions.east, Rect::new(9., 2., 10., 10.));

        assert_eq!(drag_regions.south_east, Rect::new(9., 10., 10., 12.));

        assert_eq!(drag_regions.south, Rect::new(1., 10., 9., 12.));

        assert_eq!(drag_regions.south_west, Rect::new(0., 10., 1., 12.));

        assert_eq!(drag_regions.west, Rect::new(0., 2., 1., 10.));
    }
}
