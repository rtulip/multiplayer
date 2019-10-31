use specs::{Builder, World, WorldExt};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use crate::server_side::client::{ClientCollection, ClientID};
use crate::state::State;

pub enum GameState {
    PendingPlayers(u32),
    Active,
    Paused,
}

pub struct GameModel{
    pub world: World,
    pub players: ClientCollection,
}

impl GameModel {

    pub fn new() -> GameModel {

        let mut world = World::new(); 
        world.register::<components::Position>();
        world.register::<components::Velocity>();
        world.register::<components::Player>();
        world.register::<components::Drag>();

        world.insert(GameState::Active);

        world.maintain();

        let players: HashSet<ClientID> = HashSet::new();
        let players = Arc::new(Mutex::new(players));

        GameModel {
            world,   
            players,
        }

    }

    pub fn add_player(&mut self, player_id: ClientID) {

        self.world.create_entity()
            .with(components::Position{x: 0.0, y: 0.0})
            .with(components::Velocity{x: 1.0, y: 1.0})
            .with(components::Player)
            .with(components::Drag)
            .build();

        let mut players = self.players.lock().unwrap();
        if players.insert(player_id){
            println!("Player added successfully");
        } else {
            println!("Player already in HashSet");
        }
    }

}

impl State for GameModel {
    type StateEnum = GameState;
    fn change_state(&mut self, new_state: GameState) {

        let mut state = self.world.write_resource::<GameState>();
        *state = new_state;

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