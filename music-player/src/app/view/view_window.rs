use crate::app::{CosmicAppletMusic, Message};
use cosmic::{theme, Element};
use mpris::PlaybackStatus;

pub fn view_window(app: &CosmicAppletMusic, _id: cosmic::iced::window::Id) -> Element<'_, Message> {
    let cosmic::cosmic_theme::Spacing { space_s, space_m, .. } = theme::active().cosmic().spacing;

    // Album cover
    let album_cover = if let Some(ref handle) = app.album_art_handle {
        cosmic::widget::container(
            cosmic::widget::image(handle.clone())
                .width(cosmic::iced::Length::Fixed(80.0))
                .height(cosmic::iced::Length::Fixed(80.0))
                .content_fit(cosmic::iced::ContentFit::Cover)
        )
        .width(cosmic::iced::Length::Fixed(80.0))
        .height(cosmic::iced::Length::Fixed(80.0))
        .class(cosmic::theme::Container::Card)
    } else if app.player_info.art_url.is_some() {
        // Loading state
        cosmic::widget::container(
            cosmic::widget::column()
                .push(cosmic::widget::icon::from_name("image-loading-symbolic").size(32))
                .push(cosmic::widget::text::caption("Loading...").size(10))
                .spacing(4)
                .align_x(cosmic::iced::Alignment::Center)
        )
        .width(cosmic::iced::Length::Fixed(80.0))
        .height(cosmic::iced::Length::Fixed(80.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .class(cosmic::theme::Container::Card)
    } else {
        // No art available
        cosmic::widget::container(
            cosmic::widget::icon::from_name("audio-headphones-symbolic")
                .size(48)
        )
        .width(cosmic::iced::Length::Fixed(80.0))
        .height(cosmic::iced::Length::Fixed(80.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .class(cosmic::theme::Container::Card)
    };

    let song_info = cosmic::widget::column()
        .spacing(space_s)
        .push(
            cosmic::widget::text::title4(&app.player_info.title)
        )
        .push(
            cosmic::widget::text::body(&app.player_info.artist)
        );

    let info_row = cosmic::widget::row()
        .spacing(space_m)
        .push(album_cover)
        .push(song_info)
        .align_y(cosmic::iced::Alignment::Center);

    let status_icon = match app.player_info.status {
        PlaybackStatus::Playing => "media-playback-pause-symbolic",  // Show pause when playing
        PlaybackStatus::Paused => "media-playback-start-symbolic",   // Show play when paused
        PlaybackStatus::Stopped => "media-playback-start-symbolic",  // Show play when stopped
    };

    let controls = cosmic::widget::row()
        .spacing(space_m)
        .push(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name("media-skip-backward-symbolic")
            )
            .on_press(Message::Previous)
        )
        .push(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name(status_icon)
            )
            .on_press(Message::PlayPause)
        )
        .push(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name("media-skip-forward-symbolic")
            )
            .on_press(Message::Next)
        )
        .align_y(cosmic::iced::Alignment::Center);

    // Volume control
    let volume_row = cosmic::widget::row()
        .spacing(space_s)
        .push(
            cosmic::widget::icon::from_name("audio-volume-low-symbolic")
                .size(16)
        )
        .push(
            cosmic::widget::slider(0.0..=1.0, app.player_info.volume, Message::VolumeChanged)
                .step(0.01)
                .width(cosmic::iced::Length::Fill)
        )
        .push(
            cosmic::widget::icon::from_name("audio-volume-high-symbolic")
                .size(16)
        )
        .align_y(cosmic::iced::Alignment::Center);

    let content = cosmic::widget::column()
        .spacing(space_m)
        .padding(space_m)
        .push(info_row)
        .push(
            cosmic::widget::divider::horizontal::default()
        )
        .push(
            cosmic::widget::container(controls)
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill)
        )
        .push(
            cosmic::widget::divider::horizontal::default()
        )
        .push(volume_row);

    app.core
        .applet
        .popup_container(content)
        .limits(
            cosmic::iced::Limits::NONE
                .min_height(200.)
                .min_width(350.0)
                .max_width(450.0)
                .max_height(350.0),
        )
        .into()
}
