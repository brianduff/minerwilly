use bevy::prelude::*;

static TIMER_TICK: f32 = 0.07;
//static TIMER_TICK: f32 = 0.2;

pub struct TimerPlugin;

#[derive(Resource, Deref, DerefMut)]
pub struct GameTimer(Timer);

impl Plugin for TimerPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, create_timer);
    app.add_systems(PreUpdate, tick_timer);
  }
}

fn create_timer(mut commands: Commands) {
  commands.insert_resource(
    GameTimer(Timer::from_seconds(TIMER_TICK, TimerMode::Repeating))
  );
}

fn tick_timer(time: Res<Time>, mut timer: ResMut<GameTimer>) {
  timer.tick(time.delta());
}
