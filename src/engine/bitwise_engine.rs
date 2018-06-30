use engine::command::{Command, BuildingType};
use engine::geometry::Point;
use engine::settings::{GameSettings};
use engine::{GameStatus, Player, GameState};

const MAP_WIDTH: usize = 16;
const MAP_HEIGHT: usize = 8;

const MISSILE_COOLDOWN: usize = 3;

const DEFENCE_HEALTH: usize = 4; // '20' health is 4 hits

const MAX_TESLAS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitwiseGameState {
    status: GameStatus,
    player: Player,
    opponent: Player,
    player_buildings: PlayerBuildings,
    opponent_buildings: PlayerBuildings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerBuildings {
    unconstructed: Vec<UnconstructedBuilding>,
    energy_towers: [u8; MAP_HEIGHT],
    missile_towers: [[u8; MAP_HEIGHT]; MISSILE_COOLDOWN],
    defence_towers: [[u8; MAP_HEIGHT]; DEFENCE_HEALTH],
    tesla_towers: [u8; MAP_HEIGHT],
    
    missiles: [[u16; MAP_HEIGHT]; MAP_WIDTH/4],
    tesla_cooldowns: [TeslaCooldown; MAX_TESLAS],

    unoccupied: Vec<Point>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnconstructedBuilding {
    pub pos: Point,
    pub construction_time_left: u8,
    pub building_type: BuildingType
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeslaCooldown {
    pub active: bool,
    pub pos: Point,
    pub cooldown: u8
}


impl GameState for BitwiseGameState {
    fn simulate(&mut self, _settings: &GameSettings, _player_command: Command, _opponent_command: Command) -> GameStatus {
        //TODO
        self.status
    }


    fn player(&self) -> &Player { &self.player }
    fn opponent(&self) -> &Player { &self.opponent }
    fn player_has_max_teslas(&self) -> bool { self.player_buildings.count_teslas() >= MAX_TESLAS }
    fn opponent_has_max_teslas(&self) -> bool { self.opponent_buildings.count_teslas() >= MAX_TESLAS }
    fn unoccupied_player_cells(&self) -> &Vec<Point> { &self.player_buildings.unoccupied }
    fn unoccupied_opponent_cells(&self) -> &Vec<Point> { &self.opponent_buildings.unoccupied }
}

impl PlayerBuildings {
    pub fn count_teslas(&self) -> usize {
        //TODO
        2
    }
}