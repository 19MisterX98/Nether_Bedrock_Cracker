use iced::theme::TextInput;
use iced::widget::text_input;
use iced::{Border, Color, Theme};
use iced::border::Radius;

pub struct TextInputErrorStyle;

impl text_input::StyleSheet for TextInputErrorStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = style.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: Radius::from(2.0),
                width: 1.0,
                color: palette.danger.strong.color,
            },
            icon_color: palette.background.weak.text,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = style.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: Radius::from(2.0),
                width: 1.0,
                color: palette.danger.strong.color,
            },
            icon_color: palette.background.weak.text,
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        style.placeholder_color(&TextInput::Default)
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        style.value_color(&TextInput::Default)
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        style.disabled_color(&TextInput::Default)
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        style.selection_color(&TextInput::Default)
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = style.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: Radius::from(2.0),
                width: 1.0,
                color: palette.background.base.text,
            },
            icon_color: palette.background.weak.text,
        }
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        style.disabled(&TextInput::Default)
    }
}
