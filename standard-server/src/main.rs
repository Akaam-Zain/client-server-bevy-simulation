use bevy::math::Vec3;
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityState {
    entity_atrib: HashMap<u32, Vec3>,
    count: u32,
}

struct Connection {
    stream: TcpStream,
}

#[derive(Serialize, Deserialize, Debug)]
enum ConnectionType {
    Init,
    GetEntity,
}
#[derive(Serialize, Deserialize, Debug)]
struct ConnectionParams {
    status: String,
    connection_type: ConnectionType,
    data: Option<EntityState>,
    boundary: Option<(f32, f32)>,
}

fn main() {
    let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
    let mut entities: HashMap<u32, Vec3> = HashMap::new();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        loop {
            handle_client(&stream, &mut entities);
        }
    }
}

fn handle_client(mut stream: &TcpStream, entities: &mut HashMap<u32, Vec3>) {
    // Match connection type

    //Read from client
    let mut buffer = [1; 80000];
    let len = stream.read(&mut buffer).unwrap();
    let message = String::from_utf8_lossy(&mut buffer[..len]);
    let connection_request: ConnectionParams = serde_json::from_str(&message).unwrap();
    let mut entities_in_frame: EntityState = EntityState {
        entity_atrib: HashMap::new(),
        count: BALL_COUNT,
    };
    println!("Connection status {:?}", connection_request);
    println!("{:?}", connection_request.connection_type);

    //If connection created - Spawn 100 entities and pass IDs
    let mut rng = thread_rng();
    let mut translation = Vec3::new(0., 0., 0.);

    match connection_request.connection_type {
        ConnectionType::Init => {
            for i in 1..BALL_COUNT {
                //Spawn entities
                translation.x = rng.gen_range(-500. ..500.);
                translation.y = rng.gen_range(-500. ..500.);
                translation.z = 0.;
                entities.insert(i, translation);
            }

            //Write to client
            let entity_data = ConnectionParams {
                status: "Connected".to_string(),
                connection_type: ConnectionType::GetEntity,
                data: Some(EntityState {
                    entity_atrib: entities.clone(),
                    count: BALL_COUNT,
                }),
                boundary: None,
            };
            let serialized_entity_data = serde_json::to_string(&entity_data).unwrap();
            let _ = stream.write(serialized_entity_data.as_bytes());
        }
        ConnectionType::GetEntity => {
            let mut rng = thread_rng();

            println!("ENTITIESSSSS ##### {:?}", entities);
            for (entity, mut translation) in entities {
                //If it is greater keep and move them

                //250 < 500

                if translation.x < connection_request.boundary.unwrap().0
                    || translation.y < connection_request.boundary.unwrap().1
                {
                    translation.x += 1.;
                    translation.y += 1.;
                    entities_in_frame.entity_atrib.insert(*entity, *translation);
                } else {
                    translation.x = 5000.;
                    translation.y = 5000.;
                    entities_in_frame.entity_atrib.insert(*entity, *translation);
                }
            }

            let serialized_entity_data = serde_json::to_string(&entities_in_frame).unwrap();
            stream.write(serialized_entity_data.as_bytes());
        }
    }

    //Read boundary from client
    // let mut buffer = [1; 80000];
}
