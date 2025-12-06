// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    iced::{
        Background, Border, Color, Element, Length, Padding, Point, Rectangle, Size, Vector,
        alignment::{Horizontal, Vertical},
        event::{self, Event},
        mouse, overlay,
    },
    iced_core::{
        Clipboard, Layout, Renderer as IcedRenderer, Shell, layout, renderer, text::Renderer,
        widget::Tree,
    },
    widget::{Operation, Widget},
};

/// A pill-shaped widget that displays text
#[must_use]
pub struct Pill<'a, Message> {
    /// The text to display
    text: String,
    /// Background color of the pill
    color: Color,
    /// Text color
    text_color: Color,
    /// Padding inside the pill
    padding: Padding,
    /// Font size
    font_size: f32,
    /// Width
    width: Length,
    /// Height
    height: Length,
    _phantom: std::marker::PhantomData<&'a Message>,
}

impl<'a, Message> Pill<'a, Message> {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::from_rgb8(100, 100, 100),
            text_color: Color::WHITE,
            padding: Padding::from([6.0, 12.0]),
            font_size: 14.0,
            width: Length::Shrink,
            height: Length::Shrink,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the background color of the pill
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set custom padding
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set width
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set height
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

/// Convenience function to create a pill widget
pub fn pill<'a, Message>(text: impl Into<String>) -> Pill<'a, Message> {
    Pill::new(text)
}

impl<'a, Message: 'static + Clone> Widget<Message, cosmic::Theme, cosmic::Renderer>
    for Pill<'a, Message>
{
    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn diff(&mut self, _tree: &mut Tree) {
        // No children to diff
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &cosmic::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // estimate size based on text length and font size
        let estimated_char_width = self.font_size * 0.6;
        let estimated_text_width = self.text.len() as f32 * estimated_char_width;
        let estimated_text_height = self.font_size * 1.5;

        let width = estimated_text_width + self.padding.horizontal();
        let height = estimated_text_height + self.padding.vertical();

        let size = limits.resolve(self.width, self.height, Size::new(width, height));

        layout::Node::new(size)
    }

    fn operate(
        &self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &cosmic::Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |_operation| {});
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        _event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &cosmic::Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &cosmic::Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut cosmic::Renderer,
        _theme: &cosmic::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // pill background
        renderer.fill_quad(
            cosmic::iced::advanced::renderer::Quad {
                bounds,
                border: Border::default().rounded(bounds.height / 2.0),
                shadow: cosmic::iced::Shadow::default(),
            },
            Background::Color(self.color),
        );

        let text_center = Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );
        renderer.fill_text(
            cosmic::iced_core::text::Text {
                content: self.text.clone(),
                bounds: Size::new(bounds.width, bounds.height),
                size: cosmic::iced::Pixels(self.font_size),
                line_height: cosmic::iced_core::text::LineHeight::default(),
                font: cosmic::font::Font::default(),
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                shaping: cosmic::iced::advanced::text::Shaping::Basic,
                wrapping: cosmic::iced_core::text::Wrapping::None,
            },
            text_center,
            self.text_color,
            bounds,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: Layout<'_>,
        _renderer: &cosmic::Renderer,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, cosmic::Theme, cosmic::Renderer>> {
        None
    }
}

impl<'a, Message: 'static + Clone> From<Pill<'a, Message>>
    for Element<'a, Message, cosmic::Theme, cosmic::Renderer>
{
    fn from(pill: Pill<'a, Message>) -> Self {
        Self::new(pill)
    }
}
