use specs::RunNow;
use crate::game::model::GameModel;


pub struct GameController {

    pub model: GameModel,
    
}

impl GameController {

    pub fn new() -> GameController {
        
        let model = GameModel::new();
        
        GameController{
            model,
        }

    }

    pub fn dispatch(&mut self) {

        systems::HelloWorld.run_now(&self.model.world);
        systems::UpdatePos.run_now(&self.model.world);
        systems::Friction{drag: 0.05}.run_now(&self.model.world);
        
    }

}

pub mod systems{
    use specs::{System, ReadStorage, WriteStorage};
    use crate::game::model::components;
    
    pub struct HelloWorld;

    impl<'a> System<'a> for HelloWorld {
        type SystemData = (ReadStorage<'a, components::Position>,
                        ReadStorage<'a, components::Player>);

        fn run(&mut self, (pos, player): Self::SystemData) {
            use specs::Join;

            for (pos, player) in (&pos, &player).join(){
                println!("hello {:?}! You're at position {:?}", player, pos)
            }
        }
    }

    pub struct UpdatePos;

    impl<'a> System<'a> for UpdatePos {
        type SystemData = (ReadStorage<'a, components::Velocity>,
                           WriteStorage<'a, components::Position>);

        fn run(&mut self, (vel, mut pos): Self::SystemData) {
            use specs::Join;
            for (vel, pos) in (&vel, &mut pos).join() {
                pos.x += vel.x * 0.05;
                pos.y += vel.y * 0.05;
            }
        }
    }

    pub struct Friction {
        pub drag: f32,
    }

    impl<'a> System<'a> for Friction {
        type SystemData = (WriteStorage<'a, components::Velocity>,
                           ReadStorage<'a, components::Drag>);

        fn run(&mut self, (mut vel, drag): Self::SystemData) {
            use specs::Join;
            for (vel, _) in (&mut vel, &drag).join() {
                let x_drag = self.drag * vel.x * vel.x;
                let y_drag = self.drag * vel.y * vel.y;
                
                match vel.x < x_drag {
                    true => vel.x = 0.0,
                    false => vel.x -= x_drag,
                }

                match vel.y < y_drag {
                    true => vel.y = 0.0,
                    false => vel.y -= y_drag,
                }
            }
        }
    }

}