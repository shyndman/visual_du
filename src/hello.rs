use bevy::prelude::*;

#[derive(Component, Debug)]
struct Person;

#[derive(Component, Debug)]
struct Name(String);

struct GreetTimer(Timer);

pub struct FileSizedEvent;

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, true)))
            .add_event::<FileSizedEvent>()
            .add_startup_system(add_people)
            .add_system(hello_world)
            .add_system(greet_people);
    }
}

fn add_people(mut commands: Commands) {
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Elaina Proctor".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Renzo Hume".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Zayna Nieves".to_string()));
}

fn greet_people(query: Query<&Name, With<Person>>, time: Res<Time>, mut timer: ResMut<GreetTimer>) {
    // Update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        // query.get(entity)
        for name in query.iter() {
            println!("hello {}!", name.0);
        }
    }
}

fn hello_world() {
    println!("hello world!");
}
