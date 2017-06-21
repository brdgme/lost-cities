extern crate brdgme_game;
extern crate brdgme_rand_bot;
extern crate lost_cities;

use lost_cities::Game;
use brdgme_rand_bot::fuzz;

use std::io::stdout;

fn main() {
    fuzz::<Game, _>(&mut stdout());
}
