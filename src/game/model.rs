use specs::{Builder, World, WorldExt};
use std::net::TcpStream;

pub struct GameModel{
    pub world: World,
}

impl GameModel {

    pub fn new() -> GameModel {

        let mut world = World::new(); 
        world.register::<components::Position>();
        world.register::<components::Velocity>();
        world.register::<components::Player>();

        world.maintain();

        GameModel {
            world,   
        }

    }

    pub fn add_player(&mut self, socket: TcpStream) {

        self.world.create_entity()
            .with(components::Position{x: 0.0, y: 0.0})
            .with(components::Velocity{x: 0.0, y: 0.0})
            .with(components::Player{socket: Some(socket)})
            .build();

    }

}

pub mod components {

    use specs::{Component, VecStorage};
    use std::net::TcpStream;

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

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Player {
        pub socket: Option<TcpStream>,
    }


}