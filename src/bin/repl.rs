extern crate brdgme_cmd;
extern crate lost_cities;

use lost_cities::Game;
use brdgme_cmd::repl;

fn main() {
    repl::<Game>();
}
