use std::fmt;
use super::geometry::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Nothing,
    Build(Point, BuildingType),
    IronCurtain
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Command::Nothing => write!(f, ""),
            Command::Build(p, b) => write!(f, "{},{},{}", p.x(), p.y(), b as u8),
            Command::IronCurtain => write!(f, "0,0,5")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Defence = 0,
    Attack = 1,
    Energy = 2,
    Tesla = 4,
}

impl BuildingType {
    pub fn all() -> [BuildingType; 4] {
        use self::BuildingType::*;
        [Defence, Attack, Energy, Tesla]
    }

    pub fn from_u8(id: u8) -> Option<BuildingType> {
        use std::mem;
        if id <= 4 && id != 3 { Some(unsafe { mem::transmute(id) }) } else { None }
    }

}
