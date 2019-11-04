use specs::{Builder, World, WorldExt};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::server_side::client::{ClientCollection, ClientID};
use crate::state::State;

#[derive(Clone, Copy)]
pub enum GameState {
    PendingPlayers(u32),
    Active,
    Paused,
}

pub struct GameModel {
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

        world.insert(GameState::PendingPlayers(2));

        world.maintain();

        let players: HashSet<ClientID> = HashSet::new();
        let players = Arc::new(Mutex::new(players));

        GameModel { world, players }
    }

    pub fn add_player(&mut self, player_id: ClientID) {
        let state = self.get_state();
        match state {
            GameState::PendingPlayers(n) => {
                if n == 1 {
                    self.change_state(GameState::Active);
                } else {
                    self.change_state(GameState::PendingPlayers(n - 1))
                }

                self.world
                    .create_entity()
                    .with(components::Position { x: 0.0, y: 0.0 })
                    .with(components::Velocity { x: 1.0, y: 1.0 })
                    .with(components::Player)
                    .with(components::Drag)
                    .build();
                let players = Arc::clone(&self.players);
                let mut players = players.lock().unwrap();
                if players.insert(player_id) {
                    println!("Player added successfully");
                } else {
                    println!("Player already in HashSet");
                }
            }
            _ => (),
        }
    }

    pub fn get_state(&self) -> GameState {
        let state = self.world.read_resource::<GameState>();
        (*state).clone()
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

    use specs::{Component, NullStorage, VecStorage};

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
