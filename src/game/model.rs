use specs::{World, WorldExt};


pub struct GameModel{
    game_world: World,
}

impl GameModel {

    pub fn new() -> GameModel {

        GameModel {
            game_world: World::new(),
        }

    }

}

mod Components {

    use specs::{Builder, Component, ReadStorage, System, VecStorage, World, WorldExt, RunNow};
    use std::net::TcpStream;

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    struct Player {
        socket: Option<TcpStream>,
    }


}