use command::Command;
use brdgme_game::GameError;

named!(pub command<&str, Command>, alt!(draw | draw));

named!(draw<&str, Command>, chain!(
    tag_s!("draw") ,
    || { Command::Draw }
));

#[cfg(test)]
mod test {
    use super::*;
    use command::Command;
    use card::{Expedition, Value};
    use nom::IResult::*;

    #[test]
    fn command_works() {
        assert_eq!(command("PLaY y8"),
                   Done("", Command::Play((Expedition::Yellow, Value::N(8)))));
        assert_eq!(command("diSCArd bX"),
                   Done("", Command::Discard((Expedition::Blue, Value::Investment))));
        assert_eq!(command("tAKE R"), Done("", Command::Take(Expedition::Red)));
        assert_eq!(command("dRaW"), Done("", Command::Draw));
    }
}
