use bevy::prelude::*;

use crate::quest::component::{
    AcceptOffer, CloseOffer, OpenOffer, QuestId, QuestOfferDialogHost, QuestOfferList,
    QuestOfferQueue, QuestStatus, QuestTracker, QuestTrackerText, VisibleOffers,
};
use crate::quest::component::QuestCatalogValue;
use crate::resource::SceneState;
use crate::scene::widget::{ButtonVisuals, spawn_button};

pub fn setup_quest_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                top: Val::Px(44.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            DespawnOnExit(SceneState::Game),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("QUESTS"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.85, 1.0)),
            ));
            root.spawn((
                QuestTrackerText,
                Text::new(String::new()),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.9, 1.0)),
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                top: Val::Percent(42.0),
                width: Val::Px(240.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            DespawnOnExit(SceneState::Game),
        ))
        .with_children(|root| {
            root.spawn((
                QuestOfferList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
            ));
        });

    commands.spawn((
        QuestOfferDialogHost,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            top: Val::Percent(55.0),
            width: Val::Px(280.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.92)),
        DespawnOnExit(SceneState::Game),
    ));
}

pub fn update_quest_tracker(
    mut text: Single<&mut Text, With<QuestTrackerText>>,
    tracker: Res<QuestTracker>,
    catalog: Res<QuestCatalogValue>,
) {
    let mut lines = Vec::new();
    for (id, record) in &tracker.records {
        if record.progress.status != QuestStatus::Active {
            continue;
        }
        let title = catalog
            .0
            .get(id)
            .map(|def| def.title.as_str())
            .unwrap_or(record.def.title.as_str());
        lines.push(title.to_string());
        for objective in &record.def.objectives {
            lines.push(format!(
                "  {} {}/{}",
                objective.label(),
                record.objective_progress(objective.id()),
                objective.target()
            ));
        }
    }
    lines.sort();
    text.0 = if lines.is_empty() {
        "No active quests".to_string()
    } else {
        lines.join("\n")
    };
}

pub fn sync_offer_cards(
    mut commands: Commands,
    host: Single<Entity, With<QuestOfferList>>,
    visible: Res<VisibleOffers>,
    tracker: Res<QuestTracker>,
    mut rendered: Local<Vec<QuestId>>,
) {
    if *rendered == visible.0 {
        return;
    }
    *rendered = visible.0.clone();

    commands.entity(host.entity()).despawn_children();
    commands.entity(host.entity()).with_children(|parent| {
        for id in &visible.0 {
            let Some(record) = tracker.records.get(id) else {
                continue;
            };
            let label = format!("Quest: {}", record.def.title);
            spawn_button(
                parent,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(30.0),
                    padding: UiRect::all(Val::Px(6.0)),
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                label,
                11.0,
                Color::srgb(1.0, 0.85, 0.5),
                OpenOffer(id.clone()),
                on_open_offer,
                ButtonVisuals::dark(),
            );
        }
    });
}

pub fn sync_offer_dialog(
    mut commands: Commands,
    host: Single<Entity, With<QuestOfferDialogHost>>,
    queue: Res<QuestOfferQueue>,
    tracker: Res<QuestTracker>,
    mut rendered: Local<Option<QuestId>>,
) {
    if *rendered == queue.open {
        return;
    }
    *rendered = queue.open.clone();

    commands.entity(host.entity()).despawn_children();
    let Some(id) = &queue.open else {
        return;
    };
    let Some(record) = tracker.records.get(id) else {
        return;
    };

    commands.entity(host.entity()).with_children(|parent| {
        parent.spawn((
            Text::new(record.def.title.clone()),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.5)),
        ));
        parent.spawn((
            Text::new(record.def.description.clone()),
            TextFont {
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
        spawn_button(
            parent,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            "Accept",
            12.0,
            Color::WHITE,
            AcceptOffer(id.clone()),
            on_accept_offer,
            ButtonVisuals::green(),
        );
        spawn_button(
            parent,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            "Close",
            12.0,
            Color::WHITE,
            CloseOffer,
            on_close_offer,
            ButtonVisuals::dark(),
        );
    });
}

fn on_open_offer(
    click: On<PointerClick>,
    buttons: Query<&OpenOffer>,
    mut queue: ResMut<QuestOfferQueue>,
) {
    let Ok(button) = buttons.get(click.event().entity) else {
        return;
    };
    queue.open = Some(button.0.clone());
}

fn on_accept_offer(
    click: On<PointerClick>,
    buttons: Query<&AcceptOffer>,
    mut tracker: ResMut<QuestTracker>,
    mut queue: ResMut<QuestOfferQueue>,
) {
    let Ok(button) = buttons.get(click.event().entity) else {
        return;
    };
    if let Some(record) = tracker.records.get_mut(&button.0) {
        record.progress.status = QuestStatus::Active;
    }
    queue.open = None;
    queue.pending.retain(|id| id != &button.0);
}

fn on_close_offer(
    click: On<PointerClick>,
    buttons: Query<&CloseOffer>,
    mut queue: ResMut<QuestOfferQueue>,
) {
    if buttons.contains(click.event().entity) {
        queue.open = None;
    }
}
