use bevy::math::Vec3;
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

const SERVER_ADDRESS: &str = "127.0.0.1:8000";
const BALL_COUNT: u32 = 100;

#[derive(Serialize, Deserialize)]
pub struct Movement {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize)]
pub struct EntityState {
    entity_atrib: HashMap<u32, Vec3>,
    count: u32,
}

struct Connection {
    stream: TcpStream,
}

fn main() {
    let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_client(stream);
    }
}

fn handle_client(mut stream: TcpStream) {
    //Read from client
    let mut buffer = [1; 80000];
    let len = stream.read(&mut buffer).unwrap();
    let message = String::from_utf8_lossy(&mut buffer[..len]);
    let mut connection_status_deserialized: &str = serde_json::from_str(&message).unwrap();
    println!("Connection status {:?}", connection_status_deserialized);

    //If connection created - Spawn 100 entities and pass IDs
    let mut rng = thread_rng();
    let mut translation = Vec3::new(0., 0., 0.);
    let mut entities: HashMap<u32, Vec3> = HashMap::new();

    if connection_status_deserialized == "Connected" {
        for i in 1..BALL_COUNT {
            //Spawn entities
            translation.x = rng.gen_range(-500. ..500.);
            translation.y = rng.gen_range(-500. ..500.);
            translation.z = 0.;
            entities.insert(i, translation);
        }

        //Write to client
        let entity_data = EntityState {
            entity_atrib: entities.clone(),
            count: BALL_COUNT,
        };
        let serialized_entity_data = serde_json::to_string(&entity_data).unwrap();
        stream.write(serialized_entity_data.as_bytes());

        connection_status_deserialized = "";
    }

    //Read boundary from client
    let mut buffer = [1; 80000];
    let len = stream.read(&mut buffer).unwrap();
    let message = String::from_utf8_lossy(&mut buffer[..len]);
    let boundary_deserialized: f32 = serde_json::from_str(&message).unwrap();
    println!("Boundary {}", boundary_deserialized);
    println!("Hello there!");

    let mut entities_in_frame: Vec<u32> = Vec::new();

    for (mut entity, translation) in entities {
        if translation.x > boundary_deserialized {
            entities_in_frame.push(entity);
        }
    }
    let serialized_entity_data = serde_json::to_string(&entities_in_frame).unwrap();
    let _ = stream.write(serialized_entity_data.as_bytes());
}
