use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};

#[derive(Component)]
#[component(storage = "Archetypal")]
struct Marker;

fn setup(mut commands: Commands) {
    for i in 0..1_000_000 {
        commands.spawn(Marker);
        if i % 100_000 == 0 {
            println!("Spawned {i} entities.");
        }
    }
    println!("Spawned all entities.");
}

fn print(entities: Query<(), With<Marker>>) {
    println!();
    for (i, _) in entities.iter().enumerate() {
        println!("{i}");
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 1.0))))
        .add_systems(Startup, setup)
        .add_systems(Update, print)
        .run();
}