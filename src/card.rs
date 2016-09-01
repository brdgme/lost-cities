use std::fmt;
use std::collections::HashMap;

use brdgme_color;

#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Value {
    Investment,
    N(usize),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub enum Expedition {
    Red,
    Green,
    White,
    Blue,
    Yellow,
}

impl Expedition {
    pub fn color(&self) -> brdgme_color::Color {
        match *self {
            Expedition::Red => brdgme_color::RED,
            Expedition::Green => brdgme_color::GREEN,
            Expedition::White => brdgme_color::GREY,
            Expedition::Blue => brdgme_color::BLUE,
            Expedition::Yellow => brdgme_color::AMBER,
        }
    }

    fn abbrev(&self) -> String {
        match *self {
            Expedition::Red => "R".to_string(),
            Expedition::Green => "G".to_string(),
            Expedition::White => "W".to_string(),
            Expedition::Blue => "B".to_string(),
            Expedition::Yellow => "Y".to_string(),
        }
    }
}

impl fmt::Display for Expedition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.abbrev())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match *self {
                   Value::Investment => "X".to_string(),
                   Value::N(n) => format!("{}", n),
               })
    }
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

pub type Card = (Expedition, Value);

pub type Deck = Vec<Card>;

pub fn by_expedition(cards: &[Card]) -> HashMap<Expedition, Vec<Card>> {
    let mut output: HashMap<Expedition, Vec<Card>> = HashMap::new();
    for e in expeditions() {
        output.insert(e, of_expedition(cards, e));
    }
    output
}

pub fn of_expedition(cards: &[Card], expedition: Expedition) -> Vec<Card> {
    cards.iter().filter(|&&(e, _)| e == expedition).map(|c| c.to_owned()).collect()
}
