extern crate brdgme_cmd;
extern crate lost_cities;

use brdgme_cmd::repl;
use lost_cities::Game;

fn main() {
    repl(&mut Game::default());
}
