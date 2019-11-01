/// Trait to give state to structs.
pub trait State {
    type StateEnum;
    fn change_state(&mut self, new_state: Self::StateEnum);
}