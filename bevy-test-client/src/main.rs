use crate::components::Ball;
use crate::components::Velocity;
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::ui::entity;
use components::CustomID;
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

mod components;

const BALL_SPRITE: &str = "ball.png";
const SERVER_ADDRESS: &str = "localhost:8000";

struct WinSize {
    w: f32,
    h: f32,
}

struct GameTextures {
    ball: Handle<Image>,
}

struct Connection {
    stream: TcpStream,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityState {
    entity_atrib: HashMap<u32, Vec3>,
    count: u32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Client-Server Simulation".to_string(),
            width: 500.0,
            height: 500.0,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, ball_spawn_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(2.))
                .with_system(window_framing_system),
        )
        // .add_system(window_framing_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: ResMut<Windows>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let window = windows.get_primary_mut().unwrap();
    let (win_h, win_w) = (window.width(), window.height());

    //add window size resource
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    let game_textures = GameTextures {
        ball: asset_server.load(BALL_SPRITE),
    };

    commands.insert_resource(game_textures);

    //Add connection as resource
    let connection = Connection {
        stream: TcpStream::connect(SERVER_ADDRESS).unwrap(),
    };

    commands.insert_resource(connection);
}

fn ball_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut connection: ResMut<Connection>,
) {
    let mut ball_count = 1;
    let mut rng = thread_rng();

    //Connect to server and notify that client is connected
    let message = "Connected";
    let serialized = serde_json::to_string(&message).unwrap();
    let _ = connection.stream.write(serialized.as_bytes());

    //Read entities from Server
    let mut buffer = [1; 80000];
    let len = connection.stream.read(&mut buffer).unwrap();
    let message = String::from_utf8_lossy(&mut buffer[..len]);
    let deserialized_entity_state: EntityState = serde_json::from_str(&message).unwrap();

    for (entity, translation) in deserialized_entity_state.entity_atrib.iter() {
        let trans_x: f32 = rng.gen_range(-500. ..500.);
        let trans_y: f32 = rng.gen_range(-500. ..500.);
        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.ball.clone(),
                transform: Transform {
                    translation: *translation,
                    scale: Vec3::new(0.05, 0.05, 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Velocity {
                x: trans_x,
                y: trans_x,
                z: 0.,
            })
            .insert(Ball)
            .insert(CustomID(*entity));

        ball_count += 1;
    }
}

fn window_framing_system(
    mut commands: Commands,
    window_size: Res<WinSize>,
    mut connection: ResMut<Connection>,
    mut query: Query<(Entity, &mut CustomID, &mut Transform), With<Ball>>,
) {
    let boundary = window_size.h / 10.;

    //Send the boundary to server here
    println!("This is the value of boundary {}", boundary);
    let serialized_boundary = serde_json::to_string(&boundary).unwrap();
    let _ = connection.stream.write(serialized_boundary.as_bytes());

    //Read entities from Server
    let mut buffer = [1; 80000];
    let len = connection.stream.read(&mut buffer).unwrap();
    let message = String::from_utf8_lossy(&mut buffer[..len]);
    let entities_within_bounds: Vec<u32> = serde_json::from_str(&message).unwrap();

    for (entity, customid, transform) in query.iter_mut() {
        println!("This is printing {:?}", entities_within_bounds);

        if !entities_within_bounds.contains(&customid.0) {
            commands.entity(entity).despawn();
        }
    }
}
