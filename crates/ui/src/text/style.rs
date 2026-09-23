use std::sync::Arc;

use gpui::{
    App, FontWeight, HighlightStyle, Pixels, Rems, SharedString, StyleRefinement, px, rems,
};

use crate::{ActiveTheme as _, highlighter::HighlightTheme};

/// TextViewStyle used to customize the style for [`TextView`].
#[derive(Clone)]
pub struct TextViewStyle {
    /// Gap of each paragraphs, default is 1 rem.
    pub paragraph_gap: Rems,
    /// Base font size for headings, default is 14px.
    pub heading_base_font_size: Pixels,
    /// Function to calculate heading font size based on heading level (1-6).
    ///
    /// The first parameter is the heading level (1-6), the second parameter is the base font size.
    /// The second parameter is the base font size.
    pub heading_font_size: Option<Arc<dyn Fn(u8, Pixels) -> Pixels + Send + Sync + 'static>>,
    /// Function to pick the font weight of a heading by its level (1-6).
    ///
    /// Default is `None`: bold for H1, semibold for H2-H5, medium for H6.
    pub heading_font_weight: Option<Arc<dyn Fn(u8) -> FontWeight + Send + Sync + 'static>>,
    /// Space above a heading that is not the first block, default is 0.
    ///
    /// The gap below a heading stays 0.3 rem, so a gap here ties the heading
    /// to the text it introduces rather than to the text above it.
    pub heading_gap_above: Rems,
    /// Highlight theme for code blocks. Default: [`HighlightTheme::default_light()`]
    pub highlight_theme: Arc<HighlightTheme>,
    /// The style refinement for code blocks.
    pub code_block: StyleRefinement,
    /// Style refinement applied to the table container (the bordered wrapper
    /// in wrap mode, the scroll viewport in horizontal-scroll mode).
    ///
    /// Set `overflow_x: scroll` here for adaptive table layout: columns fit
    /// their content when space allows, shrink (wrapping cell text) down to a
    /// per-column floor when the frame is narrower, and below that the table
    /// scrolls horizontally instead of squeezing further, e.g.
    /// `TextViewStyle::default().table({ let mut s = StyleRefinement::default(); s.overflow.x = Some(Overflow::Scroll); s })`.
    pub table: StyleRefinement,
    /// Style refinement applied to each table cell.
    ///
    /// With the scroll layout, set `white_space: nowrap` here to keep cells
    /// on a single line — columns then never shrink and the table scrolls as
    /// soon as the content is wider than the frame.
    pub table_cell: StyleRefinement,
    /// The highlight style for inline code.
    ///
    /// Default is [`HighlightStyle::default()`], the `background_color` will
    /// fallback to `cx.theme().accent`, if it is `None`.
    pub inline_code: HighlightStyle,
    /// Font family for inline code, default is `None` (the body font).
    ///
    /// GPUI shapes a line at one font size, so inline code keeps the body
    /// size; a face whose x-height is close to the body's reads as the same
    /// size.
    pub inline_code_font_family: Option<SharedString>,
    /// Draw inline code as a chip with this corner radius, default is `None`
    /// (a flat ground exactly under the glyphs).
    ///
    /// A chip pads the span with a narrow no-break space (U+202F) on each
    /// side and paints the inline code ground as one rounded shape under the
    /// pads and the code. The pads are display-only: selection and copy never
    /// see them. Paragraphs that also hold an inline image keep the flat
    /// ground.
    pub inline_code_chip: Option<Pixels>,
    /// Indent lists into a marker column, default is `None` (the marker sits
    /// flush with the text edge and each item's text starts after it).
    ///
    /// With an indent, every item of a list puts its marker right-aligned in
    /// one column, so "9." and "10." end at the same x and the text of every
    /// item starts at the same x, with wrapped lines hanging under it. The
    /// column is the indent, or the widest marker when that is wider.
    pub list_indent: Option<Rems>,
    pub is_dark: bool,
}

impl PartialEq for TextViewStyle {
    fn eq(&self, other: &Self) -> bool {
        self.paragraph_gap == other.paragraph_gap
            && self.heading_base_font_size == other.heading_base_font_size
            && match (&self.heading_font_size, &other.heading_font_size) {
                (Some(left), Some(right)) => (1..=6).all(|level| {
                    left(level, self.heading_base_font_size)
                        == right(level, other.heading_base_font_size)
                }),
                (None, None) => true,
                _ => false,
            }
            && match (&self.heading_font_weight, &other.heading_font_weight) {
                (Some(left), Some(right)) => (1..=6).all(|level| left(level) == right(level)),
                (None, None) => true,
                _ => false,
            }
            && self.heading_gap_above == other.heading_gap_above
            && self.highlight_theme == other.highlight_theme
            && self.code_block == other.code_block
            && self.table == other.table
            && self.table_cell == other.table_cell
            && self.inline_code == other.inline_code
            && self.inline_code_font_family == other.inline_code_font_family
            && self.inline_code_chip == other.inline_code_chip
            && self.list_indent == other.list_indent
            && self.is_dark == other.is_dark
    }
}

impl Default for TextViewStyle {
    fn default() -> Self {
        Self {
            paragraph_gap: rems(1.),
            heading_base_font_size: px(14.),
            heading_font_size: None,
            heading_font_weight: None,
            heading_gap_above: rems(0.),
            highlight_theme: HighlightTheme::default_light().clone(),
            code_block: StyleRefinement::default(),
            table: StyleRefinement::default(),
            table_cell: StyleRefinement::default(),
            inline_code: HighlightStyle::default(),
            inline_code_font_family: None,
            inline_code_chip: None,
            list_indent: None,
            is_dark: false,
        }
    }
}

impl TextViewStyle {
    /// Set paragraph gap, default is 1 rem.
    pub fn paragraph_gap(mut self, gap: Rems) -> Self {
        self.paragraph_gap = gap;
        self
    }

    pub fn heading_font_size<F>(mut self, f: F) -> Self
    where
        F: Fn(u8, Pixels) -> Pixels + Send + Sync + 'static,
    {
        self.heading_font_size = Some(Arc::new(f));
        self
    }

    /// Set the font weight of a heading by its level (1-6).
    pub fn heading_font_weight<F>(mut self, f: F) -> Self
    where
        F: Fn(u8) -> FontWeight + Send + Sync + 'static,
    {
        self.heading_font_weight = Some(Arc::new(f));
        self
    }

    /// Set the space above a heading that is not the first block, default is 0.
    pub fn heading_gap_above(mut self, gap: Rems) -> Self {
        self.heading_gap_above = gap;
        self
    }

    /// Set style for code blocks.
    pub fn code_block(mut self, style: StyleRefinement) -> Self {
        self.code_block = style;
        self
    }

    /// Set style for inline code spans.
    pub fn inline_code(mut self, style: HighlightStyle) -> Self {
        self.inline_code = style;
        self
    }

    /// Set the font family for inline code, e.g. the theme's mono family.
    pub fn inline_code_font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.inline_code_font_family = Some(family.into());
        self
    }

    /// Draw inline code as a padded chip with this corner radius.
    pub fn inline_code_chip(mut self, radius: impl Into<Pixels>) -> Self {
        self.inline_code_chip = Some(radius.into());
        self
    }

    /// Indent lists into a right-aligned marker column at least this wide.
    pub fn list_indent(mut self, indent: Rems) -> Self {
        self.list_indent = Some(indent);
        self
    }

    /// Set extra style for the table container.
    ///
    /// Set `overflow_x: scroll` on the refinement for adaptive layout: cells
    /// wrap as the frame narrows, and once columns reach their minimum width
    /// the table scrolls horizontally instead of shrinking further.
    pub fn table(mut self, style: StyleRefinement) -> Self {
        self.table = style;
        self
    }

    /// Set extra style for each table cell.
    ///
    /// With the scroll table layout, `white_space: nowrap` here keeps cells
    /// on a single line and the table scrolls whenever the content is wider
    /// than the frame.
    pub fn table_cell(mut self, style: StyleRefinement) -> Self {
        self.table_cell = style;
        self
    }

    /// Returns the [`HighlightStyle`] to use for inline code,
    /// fallback `background_color` to `cx.theme().accent`, if it is `None`.
    pub(crate) fn inline_code_highlight(&self, cx: &App) -> HighlightStyle {
        let mut style = self.inline_code;
        if style.background_color.is_none() {
            style.background_color = Some(cx.theme().accent);
        }
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_layout_fingerprint_covers_callback_table_and_theme_fields() {
        let base = TextViewStyle::default();
        let heading = base.clone().heading_font_size(|_, size| size);
        assert!(heading == base.clone().heading_font_size(|_, size| size));
        assert!(heading != base.clone().heading_font_size(|_, size| size * 2.));

        let mut table = StyleRefinement::default();
        table.text.white_space = Some(gpui::WhiteSpace::Nowrap);
        assert!(base != base.clone().table_cell(table));

        let mut dark = base.clone();
        dark.is_dark = true;
        assert!(base != dark);
    }

    #[test]
    fn heading_weight_and_gap_take_part_in_the_fingerprint() {
        let base = TextViewStyle::default();
        let semibold = base.clone().heading_font_weight(|_| FontWeight::SEMIBOLD);
        assert!(semibold == base.clone().heading_font_weight(|_| FontWeight::SEMIBOLD));
        assert!(semibold != base.clone().heading_font_weight(|_| FontWeight::BOLD));
        assert!(semibold != base);
        assert!(base != base.clone().heading_gap_above(rems(0.75)));
        assert_eq!(base.heading_gap_above, rems(0.), "no gap unless asked");
    }

    #[test]
    fn cloning_preserves_the_same_heading_callback_fingerprint() {
        let style = TextViewStyle::default().heading_font_size(|_, size| size);
        assert!(style == style.clone());
    }
}
