use anyrender::PaintScene;
use imaging::PaintSink;

pub struct AnyRenderImagingPainter<'s, S>
where
    S: PaintScene,
{
    scene: &'s mut S,
}

impl<'s, S> PaintSink for AnyRenderImagingPainter<'s, S>
where
    S: PaintScene + 'static,
{
    fn push_clip(&mut self, clip: imaging::ClipRef<'_>) {
        todo!()
    }

    fn pop_clip(&mut self) {
        todo!()
    }

    fn push_group(&mut self, group: imaging::GroupRef<'_>) {
        todo!()
    }

    fn pop_group(&mut self) {
        todo!()
    }

    fn fill(&mut self, draw: imaging::FillRef<'_>) {
        todo!()
    }

    fn stroke(&mut self, draw: imaging::StrokeRef<'_>) {
        todo!()
    }

    fn glyph_run(
        &mut self,
        draw: imaging::GlyphRunRef<'_>,
        glyphs: &mut dyn Iterator<Item = imaging::record::Glyph>,
    ) {
        todo!()
    }

    fn blurred_rounded_rect(&mut self, draw: imaging::BlurredRoundedRect) {
        todo!()
    }
}
