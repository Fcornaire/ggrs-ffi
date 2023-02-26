use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Default)]
pub enum ArrowTypes {
    #[default]
    Normal = 0,
    Bomb = 1,
    SuperBomb = 2,
    Laser = 3,
    Bramble = 4,
    Drill = 5,
    Bolt = 6,
    Toy = 7,
    Feather = 8,
    Trigger = 9,
    Prism = 10,
}
