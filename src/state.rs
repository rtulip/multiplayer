pub trait State{

    type StateEnum;

    fn get_state(&mut self) -> &mut Self::StateEnum;
    fn change_state(&mut self, new_state: Self::StateEnum){
        *(self.get_state()) = new_state;
    }

}