extern crate rand;
#[macro_use]
extern crate nom;
extern crate brdgme;

mod card;
mod parser;

use rand::{thread_rng, Rng};
use brdgme::{Gamer, Log};

const INVESTMENTS: usize = 3;
const ROUNDS: usize = 3;
const START_ROUND: usize = 1;
const PLAYERS: usize = 2;
const MIN_VALUE: usize = 2;
const MAX_VALUE: usize = 10;
const HAND_SIZE: usize = 8;
const PLAYER_TEMPLATE: &'static str = include_str!("player.hbs");

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
                expedition: e,
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

    fn start_round(&mut self) -> Result<Vec<Log>, String> {
        let mut deck = initial_deck();
        let mut logs: Vec<Log> = vec![
            Log::public(format!("Starting round {}", self.round)),
        ];
        thread_rng().shuffle(deck.as_mut_slice());
        self.deck = deck;
        self.hands = vec![];
        for p in 0..PLAYERS {
            self.hands.push(vec![]);
            logs.extend(try!(self.draw(p)));
        }
        Ok(logs)
    }

    fn next_round(&mut self) -> Result<Vec<Log>, String> {
        if self.round < ROUNDS {
            self.round += 1;
            self.start_round()
        } else {
            // TODO end game log
            Ok(vec![])
        }
    }

    fn draw(&mut self, player: usize) -> Result<Vec<Log>, String> {
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
            Ok(vec![])
        }
    }

    fn player_template(self, player: usize) -> String {
        PLAYER_TEMPLATE.to_string()
    }
}

impl Gamer for Game {
    fn start(&mut self, players: usize) -> Result<Vec<Log>, String> {
        if players != PLAYERS {
            return Err("Lost Cities is for 2 players".to_string());
        }
        self.round = START_ROUND;
        self.start_round()
    }

    fn command(&mut self, _: usize, input: &[u8]) -> Result<Vec<Log>, String> {
        parser::draw_command(input);
        parser::play_command(input);
        Ok(vec![])
    }

    fn instructions(self, _: usize) -> String {
        "".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use brdgme::Gamer;

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
