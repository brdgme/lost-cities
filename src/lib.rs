#![feature(plugin)]
#![plugin(peg_syntax_ext)]

extern crate rand;
extern crate brdgme_game;
extern crate brdgme_color;
extern crate brdgme_markup;

mod card;
mod command;
mod render;

peg_file! parser("parser.peg");

use card::{Card, Expedition, Value, Deck};
use rand::{thread_rng, Rng};
use brdgme_game::{Gamer, Log};
use brdgme_game::error::GameError;
use brdgme_markup::ast::Node as N;

const INVESTMENTS: usize = 3;
pub const ROUNDS: usize = 3;
pub const START_ROUND: usize = 1;
const PLAYERS: usize = 2;
const MIN_VALUE: usize = 2;
const MAX_VALUE: usize = 10;
const HAND_SIZE: usize = 8;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Phase {
    PlayOrDiscard,
    DrawOrTake,
}

impl Default for Phase {
    fn default() -> Phase {
        Phase::PlayOrDiscard
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Game {
    pub round: usize,
    pub phase: Phase,
    pub deck: Deck,
    pub discards: Deck,
    pub hands: Vec<Deck>,
    pub scores: Vec<Vec<usize>>,
    pub expeditions: Vec<Deck>,
    pub current_player: usize,
    pub discarded_expedition: Option<Expedition>,
}

fn initial_deck() -> Vec<Card> {
    let mut deck: Deck = vec![];
    for e in card::expeditions() {
        for _ in 0..INVESTMENTS {
            deck.push((e, Value::Investment));
        }
        for v in MIN_VALUE..MAX_VALUE + 1 {
            deck.push((e, Value::N(v)));
        }
    }
    return deck;
}

impl Game {
    fn start_round(&mut self) -> Result<Vec<Log>, GameError> {
        let mut logs: Vec<Log> = vec![
            Log::public(vec![N::Text(format!("Starting round {}", self.round))]),
        ];
        // Grab a new deck and shuffle it.
        let mut deck = initial_deck();
        thread_rng().shuffle(deck.as_mut_slice());
        self.deck = deck;
        // Clear out discards, hands and expeditions.
        self.discards = vec![];
        self.hands = vec![];
        self.expeditions = vec![];
        // Initialise player hands and expedition and draw initial cards.
        for p in 0..PLAYERS {
            self.hands.push(vec![]);
            self.expeditions.push(vec![]);
            logs.extend(try!(self.draw_hand_full(p)));
        }
        self.start_turn();
        Ok(logs)
    }

    fn next_round(&mut self) -> Result<Vec<Log>, GameError> {
        if self.round < START_ROUND + ROUNDS {
            self.round += 1;
            self.start_round()
        } else {
            // TODO end game log
            Ok(vec![])
        }
    }

    fn assert_phase(&self, phase: Phase) -> Result<(), GameError> {
        if phase == self.phase {
            Ok(())
        } else {
            Err(GameError::InvalidInput("not the right phase".to_string()))
        }
    }

    pub fn draw(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::DrawOrTake));
        let logs = try!(self.draw_hand_full(player));
        self.next_phase();
        Ok(logs)
    }

    fn next_phase(&mut self) {
        match self.phase {
            Phase::PlayOrDiscard => {
                self.phase = Phase::DrawOrTake;
            }
            Phase::DrawOrTake => {
                self.next_player();
            }
        }
    }

    fn next_player(&mut self) {
        self.current_player = (self.current_player + 1) % 2;
        self.start_turn();
    }

    fn start_turn(&mut self) {
        self.phase = Phase::PlayOrDiscard;
        self.discarded_expedition = None;
    }

    pub fn take(&mut self, player: usize, expedition: Expedition) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::DrawOrTake));
        if self.discarded_expedition == Some(expedition) {
            return Err(GameError::InvalidInput("you can't take the same card you just discarded"
                .to_string()));
        }
        if let Some(index) = self.discards.iter().rposition(|&(e, _)| e == expedition) {
            let c = *try!(self.discards
                .get(index)
                .ok_or(GameError::Internal("could not find discard card".to_string())));
            try!(self.hands
                    .get_mut(player)
                    .ok_or(GameError::Internal("could not find player hand".to_string())))
                .push(c);
            self.discards.remove(index);
            self.next_phase();
            Ok(vec![Log::public(vec![
                N::Player(player),
                N::Text(" took ".to_string()),
                render::card(&c),
            ])])
        } else {
            Err(GameError::InvalidInput("there are no discarded cards for that expedition"
                .to_string()))
        }
    }

    pub fn available_discard(&self, expedition: Expedition) -> Option<Card> {
        self.discards.iter().rposition(|&(e, _)| e == expedition).map(|index| self.discards[index])
    }

    pub fn remove_player_card(&mut self, player: usize, c: Card) -> Result<(), GameError> {
        try!(self.hands
            .get_mut(player)
            .ok_or(GameError::Internal(format!("could not find player hand for player {}",
                                               player)))
            .and_then(|h| {
                let index = try!(h.iter()
                    .position(|hc| c == *hc)
                    .ok_or(GameError::InvalidInput(format!("you don't have {}",
                                                           render::card_text(&c)))));
                h.remove(index);
                Ok(())
            }));
        Ok(())
    }

    pub fn discard(&mut self, player: usize, c: Card) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::PlayOrDiscard));
        try!(self.remove_player_card(player, c));
        self.discards.push(c);
        let (e, _) = c;
        self.discarded_expedition = Some(e);
        self.next_phase();
        Ok(vec![Log::public(vec![
                N::Player(player),
                N::Text(" discarded ".to_string()),
                render::card(&c),
            ])])
    }

    fn assert_has_card(&self, player: usize, c: Card) -> Result<(), GameError> {
        try!(self.hands
            .get(player)
            .ok_or(GameError::Internal(format!("could not find player hand for player {}",
                                               player)))
            .and_then(|h| {
                h.iter()
                    .position(|hc| c == *hc)
                    .ok_or(GameError::InvalidInput(format!("you don't have {}",
                                                           render::card_text(&c))))
            }));
        Ok(())
    }

    fn highest_value_in_expedition(&self, player: usize, expedition: Expedition) -> Option<usize> {
        self.expeditions.get(player).and_then(|e| {
            e.iter()
                .filter(|&c| c.0 == expedition && c.1 != Value::Investment)
                .map(|&c| if let Value::N(n) = c.1 { n } else { 0 })
                .max()
        })
    }

    pub fn play(&mut self, player: usize, c: Card) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::PlayOrDiscard));
        try!(self.assert_has_card(player, c));
        let (e, v) = c;
        if let Some(hn) = self.highest_value_in_expedition(player, e) {
            match v {
                Value::Investment => {
                    return Err(GameError::InvalidInput(format!("you can't play {} as you've \
                                                                already played a higher card",
                                                               render::card_text(&c))));
                }
                Value::N(n) => {
                    if n <= hn {
                        return Err(GameError::InvalidInput(format!("you can't play {} as \
                                                                    you've already played a \
                                                                    higher card",
                                                                   render::card_text(&c))));
                    }
                }
            }
        }
        try!(self.remove_player_card(player, c));
        try!(self.expeditions
                .get_mut(player)
                .ok_or(GameError::Internal(format!("could not find player expedition for \
                                                    player {}",
                                                   player))))
            .push(c);
        self.next_phase();
        Ok(vec![Log::public(vec![
                N::Player(player),
                N::Text(" played ".to_string()),
                render::card(&c),
            ])])
    }

    fn draw_hand_full(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        let mut logs: Vec<Log> = vec![];
        match self.hands.get_mut(player) {
            Some(hand) => {
                let mut num = HAND_SIZE - hand.len();
                let dl = self.deck.len();
                if num > dl {
                    num = dl;
                }
                let mut drawn: card::Deck = vec![];
                for c in self.deck.drain(..num) {
                    hand.push(c);
                    drawn.push(c);
                }
                drawn.sort();
                let d_len = drawn.len();
                let mut public_log: Vec<N> = vec![
                    N::Player(player),
                    N::Text(" drew ".to_string()),
                ];
                if d_len == 1 {
                    public_log.append(&mut vec![N::Text("a card".to_string())]);
                } else {
                    public_log.append(&mut vec![
                        N::Bold(vec![N::Text(format!("{}", drawn.len()))]),
                        N::Text(" cards".to_string()),
                    ]);
                }
                public_log.append(&mut vec![
                    N::Text(", ".to_string()),
                    N::Bold(vec![N::Text(format!("{}", self.deck.len()))]),
                    N::Text(" remaining".to_string()),
                ]);
                logs.push(Log::public(public_log));
                let mut private_log: Vec<N> = vec![
                    N::Text("You drew ".to_string()),
                ];
                private_log.append(&mut render::comma_cards(&drawn));
                logs.push(Log::private(private_log, vec![player]));
            }
            None => return Err(GameError::Internal("invalid player number".to_string())),
        };
        if self.deck.len() == 0 {
            self.next_round()
        } else {
            Ok(logs)
        }
    }

    fn player_score(&self, player: usize) -> usize {
        self.scores.iter().fold(0, |acc, rs| acc + rs.get(player).unwrap_or(&0))
    }
}

pub fn opponent(player: usize) -> usize {
    (player + 1) % 2
}

impl Gamer for Game {
    fn start(&mut self, players: usize) -> Result<Vec<Log>, GameError> {
        if players != PLAYERS {
            return Err(GameError::PlayerCount(2, 2, players));
        }
        self.round = START_ROUND;
        self.start_round()
    }

    fn is_finished(&self) -> bool {
        self.round >= START_ROUND + ROUNDS
    }

    fn winners(&self) -> Vec<usize> {
        if !self.is_finished() {
            return vec![];
        }
        let p0_score = self.player_score(0);
        let p1_score = self.player_score(1);
        if p0_score > p1_score {
            vec![0]
        } else if p1_score > p0_score {
            vec![1]
        } else {
            vec![0, 1]
        }
    }

    fn whose_turn(&self) -> Vec<usize> {
        vec![self.current_player]
    }
}

impl From<parser::ParseError> for GameError {
    fn from(err: parser::ParseError) -> GameError {
        use std::error::Error;
        GameError::InvalidInput(err.description().to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::card::{Expedition, Value};
    use brdgme_game::Gamer;

    fn discard_and_draw(game: &mut Game, player: usize) {
        let c = game.hands[player][0];
        game.discard(player, c).unwrap();
        game.draw(player).unwrap();
    }

    #[test]
    fn start_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        assert_eq!(game.hands.len(), 2);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }

    #[test]
    fn next_round_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        for _ in 0..22 {
            let c = game.hands[0][0];
            game.discard(0, c).unwrap();
            game.draw(0).unwrap();
            let c = game.hands[1][0];
            game.discard(1, c).unwrap();
            assert_eq!(START_ROUND, game.round);
            game.draw(1).unwrap();
        }
        assert_eq!(START_ROUND + 1, game.round);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }

    #[test]
    fn game_end_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        for _ in 0..(44 * ROUNDS) {
            let p = game.current_player;
            let c = game.hands[p][0];
            game.discard(p, c).unwrap();
            game.draw(p).unwrap();
        }
        assert_eq!(game.is_finished(), true);
    }

    #[test]
    fn play_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        game.hands[0] = vec![
            (Expedition::Green, Value::Investment),
            (Expedition::Green, Value::Investment),
            (Expedition::Green, Value::N(2)),
            (Expedition::Green, Value::N(3)),
            (Expedition::Yellow, Value::Investment),
            (Expedition::Yellow, Value::Investment),
            (Expedition::Yellow, Value::N(2)),
            (Expedition::Yellow, Value::N(3)),
        ];
        game.play(0, (Expedition::Green, Value::Investment))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        game.play(0, (Expedition::Green, Value::N(2)))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        // Shouldn't be able to play GX now.
        assert!(game.play(0, (Expedition::Green, Value::Investment))
            .is_err());
        game.play(0, (Expedition::Green, Value::N(3)))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        game.play(0, (Expedition::Yellow, Value::N(3)))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        // Shouldn't be able to play Y2 now.
        assert!(game.play(0, (Expedition::Yellow, Value::N(2)))
            .is_err());
    }
}
