use crate::app::{CosmicAppletMusic, Message, PopupTab};
use cosmic::{theme, Element};
use mpris::PlaybackStatus;

pub fn view_window(app: &CosmicAppletMusic, _id: cosmic::iced::window::Id) -> Element<'_, Message> {
    let cosmic::cosmic_theme::Spacing {
        space_s, space_m, ..
    } = theme::active().cosmic().spacing;

    // Tab bar with proper alignment
    let controls_button = cosmic::widget::button::text(if app.active_tab == PopupTab::Controls {
        "● Controls"
    } else {
        "○ Controls"
    })
    .on_press(Message::SwitchTab(PopupTab::Controls));

    let settings_button = cosmic::widget::button::text(if app.active_tab == PopupTab::Settings {
        "● Settings"
    } else {
        "○ Settings"
    })
    .on_press(Message::SwitchTab(PopupTab::Settings));

    let tabs = cosmic::widget::row()
        .width(cosmic::iced::Length::Fill)
        .push(controls_button)
        .push(
            cosmic::widget::container(cosmic::widget::horizontal_space())
                .width(cosmic::iced::Length::Fill),
        )
        .push(settings_button);

    // Tab content
    let tab_content = match app.active_tab {
        PopupTab::Controls => view_controls_tab(app, space_s.into(), space_m.into()),
        PopupTab::Settings => view_settings_tab(app, space_s.into(), space_m.into()),
    };

    let content = cosmic::widget::column()
        .spacing(space_s)
        .padding(space_m)
        .push(tabs)
        .push(cosmic::widget::divider::horizontal::default())
        .push(tab_content);

    app.core
        .applet
        .popup_container(content)
        .limits(
            cosmic::iced::Limits::NONE
                .min_height(350.)
                .min_width(400.0)
                .max_width(500.0)
                .max_height(600.0),
        )
        .into()
}

fn view_controls_tab(app: &CosmicAppletMusic, space_s: f32, space_m: f32) -> Element<'_, Message> {
    // Check if no player is selected
    let no_player_selected = app
        .config_manager
        .as_ref()
        .and_then(|config| config.get_selected_player())
        .is_none();

    if no_player_selected {
        return cosmic::widget::container(
            cosmic::widget::column()
                .spacing(space_s)
                .push(cosmic::widget::icon::from_name("audio-headphones-symbolic").size(48))
                .push(cosmic::widget::text::body("No player selected"))
                .push(cosmic::widget::text::caption(
                    "Go to Settings tab to select a media player",
                ))
                .align_x(cosmic::iced::Alignment::Center),
        )
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fixed(200.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .into();
    }

    // Album cover
    let album_cover = if let Some(ref handle) = app.album_art_handle {
        cosmic::widget::container(
            cosmic::widget::image(handle.clone())
                .width(cosmic::iced::Length::Fixed(80.0))
                .height(cosmic::iced::Length::Fixed(80.0))
                .content_fit(cosmic::iced::ContentFit::Cover),
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
                .align_x(cosmic::iced::Alignment::Center),
        )
        .width(cosmic::iced::Length::Fixed(80.0))
        .height(cosmic::iced::Length::Fixed(80.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .class(cosmic::theme::Container::Card)
    } else {
        // No art available
        cosmic::widget::container(
            cosmic::widget::icon::from_name("audio-headphones-symbolic").size(48),
        )
        .width(cosmic::iced::Length::Fixed(80.0))
        .height(cosmic::iced::Length::Fixed(80.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .class(cosmic::theme::Container::Card)
    };

    let song_info = cosmic::widget::column()
        .spacing(space_s)
        .push(cosmic::widget::text::title4(&app.player_info.title))
        .push(cosmic::widget::text::body(&app.player_info.artist));

    let info_row = cosmic::widget::row()
        .spacing(space_m)
        .push(album_cover)
        .push(song_info)
        .align_y(cosmic::iced::Alignment::Center);

    let status_icon = match app.player_info.status {
        PlaybackStatus::Playing => "media-playback-pause-symbolic", // Show pause when playing
        PlaybackStatus::Paused => "media-playback-start-symbolic",  // Show play when paused
        PlaybackStatus::Stopped => "media-playback-start-symbolic", // Show play when stopped
    };

    let controls = cosmic::widget::row()
        .spacing(space_m)
        .push(
            cosmic::widget::button::icon(cosmic::widget::icon::from_name(
                "media-skip-backward-symbolic",
            ))
            .on_press(Message::Previous),
        )
        .push(
            cosmic::widget::button::icon(cosmic::widget::icon::from_name(status_icon))
                .on_press(Message::PlayPause),
        )
        .push(
            cosmic::widget::button::icon(cosmic::widget::icon::from_name(
                "media-skip-forward-symbolic",
            ))
            .on_press(Message::Next),
        )
        .align_y(cosmic::iced::Alignment::Center);

    // Volume control
    let volume_row = cosmic::widget::row()
        .spacing(space_s)
        .push(cosmic::widget::icon::from_name("audio-volume-low-symbolic").size(16))
        .push(
            cosmic::widget::slider(0.0..=1.0, app.player_info.volume, Message::VolumeChanged)
                .step(0.01)
                .width(cosmic::iced::Length::Fill),
        )
        .push(cosmic::widget::icon::from_name("audio-volume-high-symbolic").size(16))
        .align_y(cosmic::iced::Alignment::Center);

    cosmic::widget::column()
        .spacing(space_m)
        .push(info_row)
        .push(cosmic::widget::divider::horizontal::default())
        .push(
            cosmic::widget::container(controls)
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill),
        )
        .push(cosmic::widget::divider::horizontal::default())
        .push(volume_row)
        .into()
}

fn view_settings_tab(app: &CosmicAppletMusic, _space_s: f32, space_m: f32) -> Element<'_, Message> {
    // Get discovered players
    let discovered_players = app.music_controller.get_discovered_players();

    let mut settings_content = cosmic::widget::column().spacing(space_m);

    // Auto-detect section
    if let Some(ref config) = app.config_manager {
        let auto_detect_enabled = config.get_auto_detect_new_players();

        let auto_detect_checkbox =
            cosmic::widget::checkbox("Auto-detect new players", auto_detect_enabled)
                .on_toggle(Message::ToggleAutoDetect);

        settings_content = settings_content.push(auto_detect_checkbox);
    }

    // Discover Players button
    let discover_button = cosmic::widget::button::text("Discover Players")
        .on_press(Message::DiscoverPlayers)
        .width(cosmic::iced::Length::Fill);

    settings_content = settings_content.push(discover_button);

    // Player selection section
    settings_content = settings_content.push(cosmic::widget::text::title4("Player Selection"));

    settings_content = settings_content.push(cosmic::widget::text::caption(
        "Choose which media player to control:",
    ));

    // Get currently selected player
    let current_selected = if let Some(ref config) = app.config_manager {
        config.get_selected_player()
    } else {
        None
    };

    let selected_index = current_selected
        .as_ref()
        .and_then(|selected| {
            discovered_players
                .iter()
                .position(|p| &p.identity == selected)
        })
        .map(|idx| idx + 1)
        .or(if current_selected.is_none() {
            Some(0)
        } else {
            None
        });

    // "None" option to disable all players
    let none_radio =
        cosmic::widget::radio("None (disable all players)", 0usize, selected_index, |_| {
            Message::SelectPlayer(None)
        });
    settings_content = settings_content.push(none_radio);

    // Add radio buttons for each discovered player
    for (index, player) in discovered_players.iter().enumerate() {
        let status_text = if player.is_active {
            " (♪ currently playing)"
        } else {
            ""
        };
        let radio_text = format!("{}{}", player.identity, status_text);

        let radio = cosmic::widget::radio(
            cosmic::widget::text::body(radio_text),
            index + 1,
            selected_index,
            {
                let player_name = player.identity.clone();
                move |_| Message::SelectPlayer(Some(player_name.clone()))
            },
        );
        settings_content = settings_content.push(radio);
    }

    if discovered_players.is_empty() {
        settings_content = settings_content.push(cosmic::widget::text::caption(
            "No players discovered yet. Click 'Discover Players' to search.",
        ));
    }

    cosmic::widget::scrollable(settings_content).into()
}
