use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
pub struct SpinnerPlugin;

impl Plugin for SpinnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SpinnerState>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    increment_atlas_index.run_if(in_state(SpinnerState::On)),
                    toggle_spinner.run_if(state_changed::<SpinnerState>),
                ),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum SpinnerState {
    #[default]
    Off,
    On,
}

#[derive(Component)]
struct Spinner;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("ui/spinner.png");
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::splat(23), 4, 2, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn((
            NodeBundle {
                z_index: ZIndex::Global(3),
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::End,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            Spinner,
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(46.),
                        height: Val::Px(46.),
                        margin: UiRect::all(Val::Px(32.)),
                        ..default()
                    },
                    image: UiImage::new(texture_handle),
                    ..default()
                },
                TextureAtlas::from(texture_atlas_handle),
                Pickable::IGNORE,
            ));
        });
}

fn increment_atlas_index(
    mut atlas_images: Query<&mut TextureAtlas>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time<Real>>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.05, TimerMode::Repeating));
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    for mut atlas_image in &mut atlas_images {
        atlas_image.index = (atlas_image.index + 1) % 8;
    }
}

fn toggle_spinner(
    state: Res<State<SpinnerState>>,
    mut query: Query<&mut Visibility, With<Spinner>>,
) {
    *query.single_mut() = if *state == SpinnerState::On {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}
