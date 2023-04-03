use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum PickupState {
    Arrows = 0,
    BombArrows = 1,
    SuperBombArrows = 2,
    LaserArrows = 3,
    BrambleArrows = 4,
    DrillArrows = 5,
    BoltArrows = 6,
    FeatherArrows = 7,
    TriggerArrows = 8,
    PrismArrows = 9,
    Shield = 10,
    Wings = 11,
    SpeedBoots = 12,
    Mirror = 13,
    TimeOrb = 14,
    DarkOrb = 15,
    LavaOrb = 16,
    SpaceOrb = 17,
    ChaosOrb = 18,
    Bomb = 19,
    Gem = 20,
}
