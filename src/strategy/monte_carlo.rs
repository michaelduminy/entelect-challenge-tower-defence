use engine::settings::GameSettings;
use engine::command::*;
use engine::geometry::*;
use engine::{GameState, GameStatus};

use rand::{Rng, XorShiftRng, SeedableRng};
    
const MAX_MOVES: u16 = 400;

use time::{Duration, PreciseTime};

#[cfg(not(feature = "single-threaded"))]
use rayon::prelude::*;

pub fn choose_move(settings: &GameSettings, state: &GameState, start_time: &PreciseTime, max_time: Duration) -> Command {
    let mut command_scores = CommandScore::init_command_scores(settings, state);
    
    loop {
        #[cfg(feature = "single-threaded")]
        {
            command_scores.iter_mut()
                .for_each(|score| {
                    let mut rng = XorShiftRng::from_seed(score.next_seed);
                    simulate_to_endstate(score, settings, state, &mut rng);
                });
        }
        #[cfg(not(feature = "single-threaded"))]
        {
            command_scores.par_iter_mut()
                .for_each(|score| {
                    let mut rng = XorShiftRng::from_seed(score.next_seed);
                    simulate_to_endstate(score, settings, state, &mut rng);
                });
        }
        if start_time.to(PreciseTime::now()) > max_time {
            break;
        }
    }

    let command = command_scores.iter().max_by_key(|&c| c.win_ratio());

    #[cfg(feature = "benchmarking")]
    {
        let total_iterations: u32 = command_scores.iter().map(|c| c.attempts).sum();
        println!("Iterations: {}", total_iterations);
    }
    
    match command {
        Some(ref command) => command.command,
        _ => Command::Nothing
    }
}

fn simulate_to_endstate<R: Rng>(command_score: &mut CommandScore, settings: &GameSettings, state: &GameState, rng: &mut R) {
    let opponent_first = random_opponent_move(settings, state, rng);
    let mut state_mut = state.simulate(settings, command_score.command, opponent_first);
    
    for _ in 0..MAX_MOVES {
        if state_mut.status != GameStatus::Continue {
            break;
        }

        let player_command = random_player_move(settings, &state_mut, rng);
        let opponent_command = random_opponent_move(settings, &state_mut, rng);
        state_mut.simulate_mut(settings, player_command, opponent_command);
    }

    let next_seed = [rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32()];
    match state_mut.status {
        GameStatus::PlayerWon => command_score.add_victory(next_seed),
        GameStatus::OpponentWon => command_score.add_defeat(next_seed),
        GameStatus::Continue => command_score.add_stalemate(next_seed),
        GameStatus::Draw => command_score.add_draw(next_seed)
    }
}

fn random_player_move<R: Rng>(settings: &GameSettings, state: &GameState, rng: &mut R) -> Command {
    let all_buildings = state.player_affordable_buildings(settings);
    random_move(&state.unoccupied_player_cells, &all_buildings, rng)
}

fn random_opponent_move<R: Rng>(settings: &GameSettings, state: &GameState, rng: &mut R) -> Command {
    let all_buildings = state.opponent_affordable_buildings(settings);
    random_move(&state.unoccupied_opponent_cells, &all_buildings, rng)
}

fn random_move<R: Rng>(all_positions: &[Point], all_buildings: &[BuildingType], rng: &mut R) -> Command {
    let number_of_commands = all_positions.len()*all_buildings.len()+1;
    let choice_index = rng.gen_range(0, number_of_commands);

    if choice_index == number_of_commands - 1 {
        Command::Nothing
    } else {
        Command::Build(
            all_positions[choice_index/all_buildings.len()],
            all_buildings[choice_index%all_buildings.len()]
        )
    }
}

#[derive(Debug)]
struct CommandScore {
    command: Command,
    victories: u32,
    defeats: u32,
    draws: u32,
    stalemates: u32,
    attempts: u32,
    next_seed: [u32; 4]
}

impl CommandScore {
    fn new(command: Command) -> CommandScore {
        CommandScore {
            command: command,
            victories: 0,
            defeats: 0,
            draws: 0,
            stalemates: 0,
            attempts: 0,
            next_seed: [0x7b6ae1f4, 0x413ce90f, 0x67816799, 0x770a6bda]
        }
    }

    fn add_victory(&mut self, next_seed: [u32; 4]) {
        self.victories += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_defeat(&mut self, next_seed: [u32; 4]) {
        self.defeats += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_draw(&mut self, next_seed: [u32; 4]) {
        self.draws += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_stalemate(&mut self, next_seed: [u32; 4]) {
        self.stalemates += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn win_ratio(&self) -> u32 {
        self.victories * 1000 / self.attempts
    }
    
    fn init_command_scores(settings: &GameSettings, state: &GameState) -> Vec<CommandScore> {
        let all_buildings = state.player_affordable_buildings(settings);

        let mut commands = Vec::with_capacity(state.unoccupied_player_cells.len()*all_buildings.len()+1);
        commands.push(CommandScore::new(Command::Nothing));

        for &position in &state.unoccupied_player_cells {
            for &building in &all_buildings {
                commands.push(CommandScore::new(Command::Build(position, building)));
            }
        }

        commands
    }
}
