pub trait state {

    type StateEnum;
    fn change_state(&mut self, new_state: Self::StateEnum);

}