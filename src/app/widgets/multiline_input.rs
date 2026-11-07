// SPDX-License-Identifier: GPL-3.0

use cosmic::cosmic_theme::palette::WithAlpha;
use cosmic::iced::core::text::{LineHeight, Wrapping};
use cosmic::iced::{Background, Border, Color, Length};
use cosmic::prelude::ColorExt;
use cosmic::widget::text_editor::{self, Content, Status, Style};
use cosmic::{Element, Theme, theme};

const TEXT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 20.0;
const BORDER_WIDTH: f32 = 2.0;
const CHARS_PER_LINE: usize = 50;

/// Multi-line text input
pub fn multiline_input<'a, Message: Clone + 'static>(
    content: &'a Content,
    placeholder: String,
    min_height: f32,
    max_height: f32,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
) -> Element<'a, Message> {
    let padding = theme::active().cosmic().spacing.space_xxs;
    let height = content_height(content, f32::from(padding)).clamp(min_height, max_height);

    text_editor::text_editor(content)
        .placeholder(placeholder)
        .on_action(on_action)
        .padding(padding)
        .size(TEXT_SIZE)
        .line_height(LineHeight::Absolute(LINE_HEIGHT.into()))
        .wrapping(Wrapping::WordOrGlyph)
        .height(Length::Fixed(height))
        .style(style)
        .into()
}

fn content_height(content: &Content, padding: f32) -> f32 {
    let visual_lines: usize = content
        .lines()
        .map(|line| line.text.chars().count().div_ceil(CHARS_PER_LINE).max(1))
        .sum();

    visual_lines.max(1) as f32 * LINE_HEIGHT + 2.0 * (padding + BORDER_WIDTH)
}

fn style(theme: &Theme, status: Status) -> Style {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let background: Color = container.small_widget.with_alpha(0.25).into();
    let placeholder = Color::from(container.on).blend_alpha(background, 0.7);

    let border_color: Color = match status {
        Status::Active | Status::Disabled => container.component.divider.into(),
        Status::Hovered | Status::Focused { .. } => cosmic.accent.base.into(),
    };

    Style {
        background: Background::Color(background),
        border: Border {
            radius: cosmic.corner_radii.radius_s.into(),
            width: BORDER_WIDTH,
            color: border_color,
        },
        placeholder,
        value: container.on.into(),
        selection: cosmic.accent.base.with_alpha(0.4).into(),
    }
}
