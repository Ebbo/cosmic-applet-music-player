use crate::app::{CosmicAppletMusic, Message};
use cosmic::app::Core;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::theme::Button;
use cosmic::widget::Id;
use cosmic::Element;
use mpris::PlaybackStatus;
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::LazyLock;

pub mod view_window;

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

#[allow(dead_code)]
pub enum AppIcon {
    Playing,
    Paused,
    Stopped,
    NoPlayer,
}

impl AppIcon {
    fn to_str(&self) -> &'static str {
        match self {
            AppIcon::Playing => "media-playback-start-symbolic",
            AppIcon::Paused => "media-playback-pause-symbolic",
            AppIcon::Stopped => "media-playback-stop-symbolic",
            AppIcon::NoPlayer => "audio-headphones-symbolic",
        }
    }
}

pub fn view(app: &CosmicAppletMusic) -> Element<'_, Message> {
    let icon = match app.player_info.status {
        PlaybackStatus::Playing => AppIcon::Paused,  // Show pause when playing
        PlaybackStatus::Paused => AppIcon::Playing,  // Show play when paused
        PlaybackStatus::Stopped => AppIcon::Playing, // Show play when stopped
    };

    use cosmic::iced::mouse;

    cosmic::widget::autosize::autosize(
        cosmic::widget::mouse_area(
            app.core
                .applet
                .icon_button(icon.to_str())
                .on_press_down(Message::TogglePopup)
        )
        .on_scroll(|delta| {
            match delta {
                mouse::ScrollDelta::Lines { y, .. } => {
                    if y > 0.0 {
                        Message::ScrollUp
                    } else {
                        Message::ScrollDown
                    }
                }
                mouse::ScrollDelta::Pixels { y, .. } => {
                    if y > 0.0 {
                        Message::ScrollUp
                    } else {
                        Message::ScrollDown
                    }
                }
            }
        })
        .on_middle_press(Message::MiddleClick),
        AUTOSIZE_MAIN_ID.clone(),
    )
    .into()
}

#[allow(dead_code)]
pub fn applet_icon(core: &Core, icon_type: AppIcon) -> cosmic::widget::Icon {
    // Hardcode to symbolic = true.
    let suggested = core.applet.suggested_size(true);

    let icon = cosmic::widget::icon::from_name(icon_type.to_str())
        .symbolic(true)
        .size(suggested.0)
        .into();
    cosmic::widget::icon(icon)
        .class(cosmic::theme::Svg::Custom(Rc::new(|theme| {
            cosmic::widget::svg::Style {
                color: Some(theme.cosmic().background.on.into()),
            }
        })))
        .width(Length::Fixed(suggested.0 as f32))
        .height(Length::Fixed(suggested.1 as f32))
}

#[allow(dead_code)]
pub fn applet_button_with_text<'a, Message: 'static + Clone>(
    core: &Core,
    icon: AppIcon,
    text: impl Into<Cow<'a, str>>,
) -> cosmic::widget::Button<'a, Message> {
    let (configured_width, _) = core.applet.suggested_window_size();

    let icon = applet_icon(core, icon);
    let text = core
        .applet
        .text(text)
        .wrapping(cosmic::iced_core::text::Wrapping::Glyph);

    let container = if core.applet.is_horizontal() {
        cosmic::widget::layer_container(
            cosmic::widget::row::with_children(vec![icon.into(), text.into()])
                .align_y(cosmic::iced::Alignment::Center)
                .spacing(4),
        )
    } else {
        cosmic::widget::layer_container(
            cosmic::widget::column::with_children(vec![icon.into(), text.into()])
                .align_x(cosmic::iced::Alignment::Center)
                .max_width(configured_width.get() as f32)
                .spacing(2),
        )
    }
    .align_x(Horizontal::Center.into())
    .align_y(Vertical::Center.into());
    cosmic::widget::button::custom(container).class(Button::AppletIcon)
}
