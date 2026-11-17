use bevy::prelude::*;

use crate::resource::AppNotice;
use crate::simulation::SimulationSet;

pub struct NoticePlugin;

impl Plugin for NoticePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppNotice>()
            .add_systems(Update, (update_notice_ui, tick_notice).chain().in_set(SimulationSet::Apply));
    }
}

#[derive(Component)]
struct NoticeRoot;

#[derive(Component)]
struct NoticeText;

fn update_notice_ui(
    mut commands: Commands,
    notice: Res<AppNotice>,
    root: Query<Entity, With<NoticeRoot>>,
    mut text: Query<&mut Text, With<NoticeText>>,
) {
    if notice.message.is_empty() {
        for entity in &root {
            commands.entity(entity).despawn();
        }
        return;
    }

    if let Some(mut text) = text.iter_mut().next() {
        **text = notice.message.clone();
        return;
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(64.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9)),
            NoticeRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(notice.message.clone()),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.4)),
                Node {
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                NoticeText,
            ));
        });
}

fn tick_notice(time: Res<Time>, mut notice: ResMut<AppNotice>) {
    if notice.message.is_empty() {
        return;
    }
    notice.remaining = (notice.remaining - time.delta_secs()).max(0.0);
    if notice.remaining <= 0.0 {
        notice.message.clear();
    }
}
