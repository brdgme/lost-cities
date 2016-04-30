#[derive(Copy, Clone)]
pub enum Value {
    Investment,
    N(usize),
}

#[derive(Copy, Clone)]
pub enum Expedition {
    Red,
    Green,
    White,
    Blue,
    Yellow,
}

pub fn expeditions() -> Vec<Expedition> {
    vec![
        Expedition::Red,
        Expedition::Green,
        Expedition::White,
        Expedition::Blue,
        Expedition::Yellow,
    ]
}

pub struct Card {
    pub expedition: Expedition,
    pub value: Value,
}

pub type Deck = Vec<Card>;
