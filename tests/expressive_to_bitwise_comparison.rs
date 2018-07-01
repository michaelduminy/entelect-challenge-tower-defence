extern crate zombot;

#[macro_use] extern crate proptest;
extern crate rand;

use zombot::input;
use zombot::engine::command::{Command, BuildingType};
use zombot::engine::geometry::Point;
use zombot::engine::settings::GameSettings;
use zombot::engine::{GameState, GameStatus, Player};

use zombot::engine::expressive_engine;
use zombot::engine::bitwise_engine;

use proptest::prelude::*;

use rand::{Rng, XorShiftRng, SeedableRng};


const STATE_PATH: &str = "tests/state0.json";

#[test]
fn reads_into_bitwise_correctly() {
    let (_, expressive_state) = input::json::read_expressive_state_from_file(STATE_PATH).expect("Failed to load expressive state");
    let bitwise_state = input::json::read_bitwise_state_from_file(STATE_PATH).expect("Failed to load bitwise state");

    assert_eq!(build_bitwise_from_expressive(&expressive_state), bitwise_state.clone());
}

proptest! {
    #[test]
    #[ignore]
    fn follows_the_same_random_game_tree(seed in any::<[u32;4]>()) {
        let mut rng = XorShiftRng::from_seed(seed);
        
        let (settings, mut expressive_state) = input::json::read_expressive_state_from_file(STATE_PATH).expect("Failed to load expressive state");
        let mut bitwise_state = input::json::read_bitwise_state_from_file(STATE_PATH).expect("Failed to load bitwise state");

        let mut expected_status = GameStatus::Continue;
        while expected_status == GameStatus::Continue {
            let player_command = random_player_move(&settings, &expressive_state, &mut rng);
            let opponent_command = random_opponent_move(&settings, &expressive_state, &mut rng);

            expected_status = expressive_state.simulate(&settings, player_command, opponent_command);
            let actual_status = bitwise_state.simulate(&settings, player_command, opponent_command);

            assert_eq!(&expected_status, &actual_status);
            assert_eq!(build_bitwise_from_expressive(&expressive_state), bitwise_state.clone());
        }
    }
}



fn random_player_move<R: Rng, GS: GameState>(settings: &GameSettings, state: &GS, rng: &mut R) -> Command {
    let all_buildings = sensible_buildings(settings, &state.player(), state.player_has_max_teslas());
    random_move(&state.unoccupied_player_cells(), &all_buildings, rng)
}

fn random_opponent_move<R: Rng, GS: GameState>(settings: &GameSettings, state: &GS, rng: &mut R) -> Command {
    let all_buildings = sensible_buildings(settings, &state.opponent(), state.opponent_has_max_teslas());
    random_move(&state.unoccupied_opponent_cells(), &all_buildings, rng)
}

fn random_move<R: Rng>(free_positions: &[Point], all_buildings: &[BuildingType], rng: &mut R) -> Command {
    
    let building_command_count = free_positions.len()*all_buildings.len();
    let nothing_count = 1;

    let number_of_commands = building_command_count + nothing_count;
    
    let choice_index = rng.gen_range(0, number_of_commands);

    if choice_index == number_of_commands - 1 {
        Command::Nothing
    } else {
        Command::Build(
            free_positions[choice_index/all_buildings.len()],
            all_buildings[choice_index%all_buildings.len()]
        )
    }
}

fn sensible_buildings(settings: &GameSettings, player: &Player, has_max_teslas: bool) -> Vec<BuildingType> {
    let mut result = Vec::with_capacity(4);
    for b in BuildingType::all().iter() {
        let building_setting = settings.building_settings(*b);
        let affordable = building_setting.price <= player.energy;
        let is_tesla = *b == BuildingType::Tesla;
        if affordable && (!is_tesla || !has_max_teslas) {
            result.push(*b);
        }
    }
    result
}

fn build_bitwise_from_expressive(expressive: &expressive_engine::ExpressiveGameState) -> bitwise_engine::BitwiseGameState {
    let player_unconstructed = expressive.player_unconstructed_buildings.iter()
        .map(build_bitwise_unconstructed_from_expressive)
        .collect();
    let opponent_unconstructed = expressive.opponent_unconstructed_buildings.iter()
        .map(build_bitwise_unconstructed_from_expressive)
        .collect();
    
    let player_energy = expressive.player_buildings.iter()
        .filter(|b| identify_building_type(b.weapon_damage, b.energy_generated_per_turn) == BuildingType::Energy)
        .fold(0, |acc, next| acc | next.pos.to_left_bitfield(8));
    let opponent_energy = expressive.opponent_buildings.iter()
        .filter(|b| identify_building_type(b.weapon_damage, b.energy_generated_per_turn) == BuildingType::Energy)
        .fold(0, |acc, next| acc | next.pos.to_left_bitfield(8));

    let mut player_buildings_iter = (0..4)
        .map(|i| expressive.player_buildings.iter()
             .filter(|b| b.health >= i*5)
             .fold(0, |acc, next| acc | next.pos.to_left_bitfield(8))
        );
    let mut opponent_buildings_iter = (0..4)
        .map(|i| expressive.opponent_buildings.iter()
             .filter(|b| b.health >= i*5)
             .fold(0, |acc, next| acc | next.pos.to_left_bitfield(8))
        );

    let mut player_attack_iter = (0..4)
        .map(|i| expressive.player_buildings.iter()
             .filter(|b| identify_building_type(b.weapon_damage, b.energy_generated_per_turn) == BuildingType::Attack)
             .filter(|b| b.weapon_cooldown_time_left == i)
             .fold(0, |acc, next| acc | next.pos.to_left_bitfield(8))
        );
    let mut opponent_attack_iter = (0..4)
        .map(|i| expressive.opponent_buildings.iter()
             .filter(|b| identify_building_type(b.weapon_damage, b.energy_generated_per_turn) == BuildingType::Attack)
             .filter(|b| b.weapon_cooldown_time_left == i)
             .fold(0, |acc, next| acc | next.pos.to_left_bitfield(8))
        );

    let empty_missiles: [(u64,u64);4] = [(0,0),(0,0),(0,0),(0,0)];
    let player_missiles = expressive.player_missiles.iter()
        .fold(empty_missiles, |acc, m| {
            let (mut left, mut right) = m.pos.to_bitfield(8);
            let mut res = acc.clone();
            for mut tier in res.iter_mut() {
                let setting = (!tier.0 & left, !tier.1 & right);
                tier.0 |= setting.0;
                tier.1 |= setting.1;
                left &= !setting.0;
                right &= !setting.1;
            }
            res
        });
    let opponent_missiles = expressive.opponent_missiles.iter()
        .fold(empty_missiles, |acc, m| {
            let (mut left, mut right) = m.pos.to_bitfield(8);
            let mut res = acc.clone();
            for mut tier in res.iter_mut() {
                let setting = (!tier.0 & left, !tier.1 & right);
                tier.0 |= setting.0;
                tier.1 |= setting.1;
                left &= !setting.0;
                right &= !setting.1;
            }
            res
        });

    let null_tesla = bitwise_engine::TeslaCooldown {
        active: false,
        pos: Point::new(0,0),
        cooldown: 0
    };
    let mut player_tesla_iter = expressive.player_buildings.iter()
        .filter(|b| identify_building_type(b.weapon_damage, b.energy_generated_per_turn) == BuildingType::Tesla)
        .map(|b| bitwise_engine::TeslaCooldown {
            active: true,
            pos: b.pos,
            cooldown: b.weapon_cooldown_time_left
        });
    let mut opponent_tesla_iter = expressive.opponent_buildings.iter()
        .filter(|b| identify_building_type(b.weapon_damage, b.energy_generated_per_turn) == BuildingType::Tesla)
        .map(|b| bitwise_engine::TeslaCooldown {
            active: true,
            pos: b.pos,
            cooldown: b.weapon_cooldown_time_left
        });
    bitwise_engine::BitwiseGameState {
        status: expressive.status,
        player: expressive.player.clone(),
        opponent: expressive.opponent.clone(),
        player_buildings: bitwise_engine::PlayerBuildings {
            unconstructed: player_unconstructed,
            buildings: [player_buildings_iter.next().unwrap(), player_buildings_iter.next().unwrap(), player_buildings_iter.next().unwrap(), player_buildings_iter.next().unwrap()],
            energy_towers: player_energy,
            missile_towers: [player_attack_iter.next().unwrap(), player_attack_iter.next().unwrap(), player_attack_iter.next().unwrap(), player_attack_iter.next().unwrap()],
            missiles: player_missiles,
            tesla_cooldowns: [player_tesla_iter.next().unwrap_or(null_tesla.clone()),
                              player_tesla_iter.next().unwrap_or(null_tesla.clone())]
        },
        opponent_buildings: bitwise_engine::PlayerBuildings {
            unconstructed: opponent_unconstructed,
            buildings: [opponent_buildings_iter.next().unwrap(), opponent_buildings_iter.next().unwrap(), opponent_buildings_iter.next().unwrap(), opponent_buildings_iter.next().unwrap()],
            energy_towers: opponent_energy,
            missile_towers: [opponent_attack_iter.next().unwrap(), opponent_attack_iter.next().unwrap(), opponent_attack_iter.next().unwrap(), opponent_attack_iter.next().unwrap()],
            missiles: opponent_missiles,
            tesla_cooldowns: [opponent_tesla_iter.next().unwrap_or(null_tesla.clone()),
                              opponent_tesla_iter.next().unwrap_or(null_tesla.clone())]
        }
    }
}

fn build_bitwise_unconstructed_from_expressive(b: &expressive_engine::UnconstructedBuilding) -> bitwise_engine::UnconstructedBuilding {
    bitwise_engine::UnconstructedBuilding {
        pos: b.pos,
        construction_time_left: b.construction_time_left,
        building_type: identify_building_type(b.weapon_damage, b.energy_generated_per_turn)
    }
}

fn identify_building_type(weapon_damage: u8, energy_generated_per_turn: u16) -> BuildingType {
    match (weapon_damage, energy_generated_per_turn) {
        (5, _) => BuildingType::Attack,
        (20, _) => BuildingType::Tesla,
        (_, 3) => BuildingType::Energy,
        _ => BuildingType::Defence
    }
}
