extern crate rand;
#[macro_use]
extern crate nom;
extern crate brdgme;

mod card;
mod parser;

use rand::{thread_rng, Rng};

const INVESTMENTS: usize = 3;
const ROUNDS: usize = 3;
const START_ROUND: usize = 1;
const PLAYERS: usize = 2;
const MIN_VALUE: usize = 2;
const MAX_VALUE: usize = 10;
const HAND_SIZE: usize = 8;


#[derive(Default)]
pub struct Game {
    pub round: usize,
    pub deck: card::Deck,
    pub hands: Vec<card::Deck>,
    pub current_player: usize,
}

fn initial_deck() -> Vec<card::Card> {
    let mut deck: card::Deck = vec![];
    for e in card::expeditions() {
        for _ in 0..INVESTMENTS {
            deck.push(card::Card {
                expedition: e,
                value: card::Value::Investment,
            });
        }
        for v in MIN_VALUE..MAX_VALUE + 1 {
            deck.push(card::Card {
                expedition: e.clone(),
                value: card::Value::N(v),
            });
        }
    }
    return deck;
}

impl Game {
    pub fn new() -> Game {
        Game::default()
    }

    fn start_round(&mut self) -> Result<(), String> {
        let mut deck = initial_deck();
        thread_rng().shuffle(deck.as_mut_slice());
        self.deck = deck;
        self.hands = vec![];
        for p in 0..PLAYERS {
            self.hands.push(vec![]);
            try!(self.draw(p));
        }
        Ok(())
    }

    fn next_round(&mut self) -> Result<(), String> {
        if self.round < ROUNDS {
            self.round += 1;
            self.start_round()
        } else {
            // TODO end game log
            Ok(())
        }
    }

    fn draw(&mut self, player: usize) -> Result<(), String> {
        match self.hands.get_mut(player) {
            Some(hand) => {
                let mut num = HAND_SIZE - hand.len();
                let dl = self.deck.len();
                if num > dl {
                    num = dl;
                }
                for c in self.deck.drain(..num) {
                    hand.push(c);
                }
            }
            None => return Err("Invalid player number".to_string()),
        };
        if self.deck.len() == 0 {
            self.next_round()
        } else {
            Ok(())
        }
    }
}

impl brdgme::Game for Game {
    fn start(&mut self, players: usize) -> Result<(), String> {
        if players != PLAYERS {
            return Err("Lost Cities is for 2 players".to_string());
        }
        self.round = START_ROUND;
        self.start_round()
    }

    fn command(&mut self, player: usize, input: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use brdgme::Game as BrdgmeGame;

    #[test]
    fn start_works() {
        let mut game = Game::new();
        assert!(game.start(2).is_ok());
        assert_eq!(game.hands.len(), 2);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }
}
