use specs::{ReadStorage, System, RunNow};
use crate::game::model::{components,GameModel};


pub struct GameController {

    pub model: GameModel,
    hello_world: HelloWorld,

}

impl GameController {

    pub fn new() -> GameController {
        
        let model = GameModel::new();
        let hello_world = HelloWorld;
        GameController{
            model,
            hello_world,
        }

    }

    pub fn dispatch(&mut self) {

        self.hello_world.run_now(&self.model.world);
        
    }

}

pub struct HelloWorld;

impl<'a> System<'a> for HelloWorld {
    type SystemData = (ReadStorage<'a, components::Position>,
                       ReadStorage<'a, components::Player>);

    fn run(&mut self, (pos, player): Self::SystemData) {
        use specs::Join;

        for (pos, player) in (&pos, &player).join(){
            println!("hello {:?}! You're at position {:?}", pos, player)
        }
    }
}