use bevy::prelude::*;
use rocketsim_rs::{
    cxx::UniquePtr,
    glam_ext::{glam::Vec3A, GameStateA},
    math::Vec3 as RVec,
    sim::{
        arena::Arena,
        ball::BallState,
        car::{CarConfig, Team},
        CarControls,
    },
};

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Car {
    pub id: u32,
    pub team: Team,
}

#[derive(Resource, Default)]
struct State(GameStateA);

pub struct RocketSimPlugin;

trait ToBevy {
    fn to_bevy(self) -> Self;
}

impl ToBevy for Vec3A {
    fn to_bevy(self) -> Self {
        Self::new(self.x, self.z, self.y)
    }
}

fn setup_arena(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut arena: NonSendMut<UniquePtr<Arena>>) {
    arena.pin_mut().add_car(Team::BLUE, CarConfig::merc());
    arena.pin_mut().add_car(Team::ORANGE, CarConfig::plank());
    arena.pin_mut().set_ball(BallState {
        pos: RVec::new(0., 0., 1500.),
        vel: RVec::new(0., 0., 1.),
        ..default()
    });

    arena.pin_mut().set_goal_scored_callback(
        |arena, _, _| {
            arena.reset_to_random_kickoff(None);
        },
        0,
    );

    arena
        .pin_mut()
        .set_all_controls(&[(1, CarControls { throttle: 1., ..default() }), (2, CarControls { throttle: 1., ..default() })])
        .unwrap();

    let game_state = arena.pin_mut().get_game_state().to_glam();

    commands.spawn((
        Ball,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: arena.get_ball_radius(),
                ..default()
            })),
            material: materials.add(StandardMaterial::from(Color::rgb(0.95, 0.16, 0.45))),
            transform: Transform::from_translation(game_state.ball.pos.to_bevy().into()),
            ..default()
        },
    ));

    for (id, team, state, config) in game_state.cars {
        let hitbox = config.hitbox_size.to_bevy();
        let color = match team {
            Team::BLUE => Color::rgb(0.4, 0.4, 0.9),
            Team::ORANGE => Color::rgb(0.9, 0.4, 0.4),
        };

        commands.spawn((
            Car { id, team },
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(hitbox.x, hitbox.y, hitbox.z))),
                material: materials.add(StandardMaterial::from(color)),
                transform: Transform::from_translation(state.pos.to_bevy().into()),
                ..default()
            },
        ));
    }
}

fn step_arena(time: Res<Time>, mut arena: NonSendMut<UniquePtr<Arena>>, mut state: ResMut<State>) {
    let current_ticks = arena.get_tick_count();
    let required_ticks = time.elapsed_seconds() * arena.get_tick_rate();
    let needs_simulation = required_ticks.floor() as u64 - current_ticks;

    if needs_simulation > 0 {
        arena.pin_mut().step(needs_simulation as i32);
        state.0 = arena.pin_mut().get_game_state().to_glam();
    }
}

fn use_game_state(state: Res<State>, mut ball: Query<&mut Transform, With<Ball>>, mut cars: Query<(&mut Transform, &Car), Without<Ball>>) {
    ball.single_mut().translation = state.0.ball.pos.to_bevy().into();

    for (mut transform, car) in cars.iter_mut() {
        let car_state = state.0.cars.iter().find(|&(id, _, _, _)| car.id == *id).unwrap().2;
        transform.translation = car_state.pos.to_bevy().into();
    }
}

impl Plugin for RocketSimPlugin {
    fn build(&self, app: &mut App) {
        rocketsim_rs::init(None);

        app.insert_non_send_resource(Arena::default_standard())
            .insert_resource(State::default())
            .add_startup_system(setup_arena)
            .add_system(step_arena.before(use_game_state))
            .add_system(use_game_state.run_if(|state: Res<State>| state.is_changed()));
    }
}
