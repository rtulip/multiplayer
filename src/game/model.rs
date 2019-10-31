use specs::{Builder, World, WorldExt};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::threading::ConnectionCollection;

pub struct GameModel{
    pub world: World,
    pub players: ConnectionCollection,
}

impl GameModel {

    pub fn new() -> GameModel {

        let mut world = World::new(); 
        world.register::<components::Position>();
        world.register::<components::Velocity>();
        world.register::<components::Player>();
        world.register::<components::Drag>();

        world.maintain();

        let players: HashMap<u32, Option<TcpStream>> = HashMap::new();
        let players = Arc::new(Mutex::new(players));

        GameModel {
            world,   
            players,
        }

    }

    pub fn add_player(&mut self, player_id: u32,socket: TcpStream) {

        self.world.create_entity()
            .with(components::Position{x: 0.0, y: 0.0})
            .with(components::Velocity{x: 1.0, y: 1.0})
            .with(components::Player)
            .with(components::Drag)
            .build();

        let mut players = self.players.lock().unwrap();
        match players.insert(player_id, Some(socket)){
            None => println!("Player added successfully"),
            _ => println!("Updated Player info"),
        }
    }

}

pub mod components {

    use specs::{Component, VecStorage, NullStorage};

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Velocity {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Component, Debug, Default)]
    #[storage(NullStorage)]
    pub struct Player;

    #[derive(Component, Default)]
    #[storage(NullStorage)]
    pub struct Drag;

}