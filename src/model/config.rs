use super::player_draw::PlayerDraw;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct Config {
    remote_addr: String,
    input_delay: i32,
    is_test_mode: bool,
    test_check_distance: i32,
    player_draw: PlayerDraw,
}

impl Config {
    pub fn new(
        remote_addr: String,
        input_delay: i32,
        is_test_mode: bool,
        test_check_distance: i32,
        player_draw: PlayerDraw,
    ) -> Self {
        Config {
            remote_addr,
            input_delay,
            is_test_mode,
            test_check_distance,
            player_draw,
        }
    }

    pub fn remote_addr(&self) -> String {
        self.remote_addr.clone()
    }

    pub fn input_delay(&self) -> i32 {
        self.input_delay
    }

    pub fn is_test_mode(&self) -> bool {
        self.is_test_mode
    }

    pub fn test_check_distance(&self) -> i32 {
        self.test_check_distance
    }

    pub fn player_draw(&self) -> PlayerDraw {
        self.player_draw
    }
}
