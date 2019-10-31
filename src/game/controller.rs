use specs::{RunNow, WorldExt};
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
        systems::Friction{drag: 0.1}.run_now(&self.model.world);

        self.model.world.maintain();

    }

}

pub mod systems{
    use specs::{System, ReadStorage, WriteStorage};
    use crate::game::model::components;
    
    pub struct HelloWorld;

    impl<'a> System<'a> for HelloWorld {
        type SystemData = (ReadStorage<'a, components::Position>,
                        ReadStorage<'a, components::Velocity>);


        fn run(&mut self, (pos, vel): Self::SystemData) {
            use specs::Join;

            for (pos, vel) in (&pos,  &vel).join(){
                println!("entity at {:?} is going {:?} km/h", pos, vel)
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
                
                match (vel.x.abs() < self.drag, vel.x > 0.0) {
                    (true, _) => vel.x = 0.0,
                    (false, true) => vel.x -= self.drag,
                    (false, false) => vel.x += self.drag, 
                }

                match (vel.y.abs() < self.drag, vel.y > 0.0) {
                    (true, _) => vel.y = 0.0,
                    (false, true) => vel.y -= self.drag,
                    (false, false) => vel.y += self.drag, 
                }
            }
        }
    }

}