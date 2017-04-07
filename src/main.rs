extern crate brdgme_cmd;

extern crate lost_cities;

use brdgme_cmd::cli::cli;
use lost_cities::Game;
use std::io::{stdin, stdout};

fn main() {
    cli::<Game, _, _>(stdin(), &mut stdout());
}
