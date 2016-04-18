extern crate rand;

use rand::{thread_rng, Rng};

const NUM_INVESTMENTS: usize = 3;
const MIN_VALUE: usize = 2;
const MAX_VALUE: usize = 10;

pub enum CardValue {
    Investment,
    Value(usize),
}

#[derive(Clone, Copy)]
pub enum Expedition {
    Red,
    Green,
    White,
    Blue,
    Yellow,
}

fn expeditions() -> Vec<Expedition> {
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
    pub value: CardValue,
}

#[derive(Default)]
pub struct Game {
    pub round: usize,
    pub deck: Vec<Card>,
}

fn initial_deck() -> Vec<Card> {
    let mut deck: Vec<Card> = vec![];
    for e in expeditions() {
        for _ in 1..NUM_INVESTMENTS {
            deck.push(Card {
                expedition: e,
                value: CardValue::Investment,
            });
        }
        for v in MIN_VALUE..MAX_VALUE {
            deck.push(Card {
                expedition: e.clone(),
                value: CardValue::Value(v),
            });
        }
    }
    return deck;
}

impl Game {
    fn start_round(&mut self) {
        let mut deck = initial_deck();
        thread_rng().shuffle(deck.as_mut_slice());
        self.deck = deck;
    }
}

impl BrdgmeGame for Game {
    fn start(&mut self, players: usize) -> Result<(), String> {
        if players != 2 {
            return Err("Lost Cities is for 2 players".to_string());
        }
        self.round = 1;
        self.start_round();
        Ok(())
    }
}

pub trait BrdgmeGame {
    fn start(&mut self, players: usize) -> Result<(), String>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_works() {
        let game = &mut Game::default() as &mut BrdgmeGame;
        assert!(game.start(2).is_ok());
    }
}
