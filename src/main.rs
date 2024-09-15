#![allow(clippy::redundant_field_names)]
use std::{cmp::Ordering, collections::{HashMap, HashSet}, fmt::Display, io, sync::{Arc, Mutex}, time::SystemTime};

fn main() {
    let jerbs: Vec<(Jobs, usize)> = vec![(Jobs::Carpenter, 1), (Jobs::Miller, 1), (Jobs::Woodcutter, 2)];
    let tiles: Vec<(TileType, usize)> = vec![(TileType::City, 1), (TileType::Forest, 6), (TileType::Plains, 3)];

    let mut world = World::generate_world(8, 5, jerbs, tiles);

    let mut exit = false;
    let mut input_buffer = String::new();
    while !exit {
        println!("\nsim step {}\n{}", world.steps, world);
        world.step_simulation();
        io::stdin().read_line(&mut input_buffer).unwrap();
        if !input_buffer.to_lowercase().contains('q') {
            input_buffer = "".to_string();
        } else {
            exit = true;
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Objects {
    Logs,
    Planks
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TileType {
    Plains,
    Forest,
    City,
}

impl Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileType::Plains => write!(f, "p"),
            TileType::Forest => write!(f, "f"),
            TileType::City => write!(f, "c"),
        }
    }
}

#[derive(Debug)]
struct Tile {
    kind: TileType,
    position: (i64, i64),
    holding: Vec<(Objects, u64)>,
}

impl Tile {
    fn check_inventory(&self, item: Objects) -> bool {
        for (tile_item, qty) in self.holding.iter() {
            if item == *tile_item && *qty > 0 {
                return true;
            }
        }
        false
    }

    fn take_item(&mut self, item: Objects, looking_for: TileType) -> bool {
        if self.kind == looking_for {
            for (tile_item, qty) in self.holding.iter_mut() {
                if item == *tile_item && *qty > 0 {
                    *qty -= 1;
                    return true;
                }
            }
        }
        false
    }

    fn deposit_item(&mut self, item: Objects) -> u64 {
        for (tile_item, qty) in self.holding.iter_mut() {
            if item == *tile_item {
                *qty += 1;
                return *qty;
            }
        }

        self.holding.push((item, 1));
        1
    }
}

fn calculate_movement(from: (i64, i64), to: (i64, i64)) -> (i64, i64) {
    let x_offset = match from.0.cmp(&to.0) {
        Ordering::Greater => -1,
        Ordering::Equal => 0,
        Ordering::Less => 1,
    };
    let y_offset = match from.1.cmp(&to.1) {
        Ordering::Greater => -1,
        Ordering::Equal => 0,
        Ordering::Less => 1,
    };
    let x = from.0 + x_offset;
    let y = from.1 + y_offset;
    (x, y)
}

fn calculate_probability<T>(integer_ratios: Vec<(T, usize)>) -> Vec<(T, f32, f32)> {
    let total_prob = integer_ratios.iter().fold(0, |total, (_, val)| total + val);
    let mut resulting_probs: Vec<(T, f32, f32)> = Vec::with_capacity(integer_ratios.len());
    let mut acc: f32 = 0.0;
    for (item, integer_prob) in integer_ratios {
        let lower = acc;
        acc += integer_prob as f32 / total_prob as f32;
        resulting_probs.push((item, lower, acc));
    }
    resulting_probs
}

struct World {
    #[allow(dead_code)]
    world_id: usize,
    steps: usize,

    world_size: (usize, usize),
    tiles: Arc<Mutex<Vec<Tile>>>,

    population: Arc<Mutex<Vec<Peepl>>>,
    job_probabilities: Vec<(Jobs, f32, f32)>,
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // size data from TILES data
        let tiles_guard = self.tiles.lock().unwrap();
        let tiles = tiles_guard.as_slice();
        let size = self.world_size;
        let width = size.0 as i64;
        let height = size.1 as i64;

        // population data from POPULATIONS data
        let pop_guard = self.population.lock().unwrap();
        let pop = pop_guard.as_slice();

        write!(f, "({}, {})\t", width, height)?;
        writeln!(f, "World population is {}: ", pop.len())?;
        for (job, min, max) in self.job_probabilities.iter() {
            let prob = ((max - min) * 100.0) as u64;
            let qty = pop.iter().fold(0, |acc, peep| {
                if *job == peep.job {
                    acc + 1
                } else {
                    acc
                }
            });
            write!(f, "{{{job:?}, {prob}%, {qty}}} ")?;
        }
        writeln!(f)?;
        for y in 0..height {
            for x in 0..width {
                write!(f, "{}", &tiles[self.get_tile_index(x, y)].kind)?;
            }
            writeln!(f)?;
        }
        

        std::fmt::Result::Ok(())
    }
}

impl World {
    fn birth_peep(position: (i64, i64), job_probabilities: &[(Jobs, f32, f32)]) -> Peepl {
        let rand_job: f32 = rand::random();
        let mut job_kind = Jobs::Woodcutter;
        for (job, lower, upper) in job_probabilities.iter() {
            if rand_job >= *lower && rand_job < *upper {
                job_kind = *job;
                break;
            }
        }
        Peepl { position: (position.0, position.1), holding: None, job: job_kind }
    }

    /// Generates a world and passes world ID that can be used as index in WORLDS global variable.
    fn generate_world(size: usize, starting_population_per_city: usize, jobs: Vec<(Jobs, usize)>, tile_makeup: Vec<(TileType, usize)>) -> World {
        // Generate world job probability table
        let world_jobs: Vec<(Jobs, f32, f32)> = calculate_probability(jobs);

        // Generate world tiles
        let tile_probs: Vec<(TileType, f32, f32)> = calculate_probability(tile_makeup);

        let mut world_tiles: Vec<Tile> = Vec::with_capacity(size*size);
        let mut world_population: Vec<Peepl> = Vec::new();
        for y in 0..size as i64 {
            for x in 0..size as i64 {
                let rand_tile: f32 = rand::random();
                let mut tile_kind = TileType::Plains;
                for (kind, lower, upper) in tile_probs.iter() {
                    if rand_tile >= *lower && rand_tile < *upper {
                        tile_kind = *kind;
                        break;
                    }
                }
                let mut holding: Vec<(Objects, u64)> = Vec::new();
                match tile_kind {
                    TileType::Plains => { },
                    TileType::Forest => holding.push((Objects::Logs, (rand::random::<f32>() * 1000.0) as u64)),
                    TileType::City => {
                        holding.push((Objects::Planks, 0));
                        for _ in 0..starting_population_per_city {
                            world_population.push(World::birth_peep((x, y), &world_jobs));
                        }
                    },
                }
                let tile = Tile {
                    kind: tile_kind,
                    position: (x, y),
                    holding: holding,
                };
                world_tiles.push(tile);
            }
        }

        World {
            world_id: 0,
            steps: 0,
            world_size: (size, size),
            tiles: Arc::new(Mutex::new(world_tiles)),
            population: Arc::new(Mutex::new(world_population)),
            job_probabilities: world_jobs,
        }
    }

    fn step_simulation(&mut self) {
        let stopwatch = SystemTime::now();

        let mut tiles_guard = self.tiles.lock().unwrap();
        let world_tiles = tiles_guard.as_mut_slice();

        let mut pop_guard = self.population.lock().unwrap();

        // Sim all population
        for peep in pop_guard.as_mut_slice() {
            let task = peep.step();
            let tile = &mut world_tiles[self.get_tile_index(peep.position.0, peep.position.1)];
            let peep_pos = (peep.position.0, peep.position.1);
            match peep.job { // TODO: Refactor code so there is less repetition to lower surface area for errors and streamline logic
                Jobs::Woodcutter => match task {
                    Action::Gather(tile_type) => {
                        if tile.take_item(Objects::Logs, tile_type) {
                            peep.holding = Some(Objects::Logs);
                            if !tile.check_inventory(Objects::Logs) {
                                tile.kind = TileType::Plains; // All trees were chopped down!
                            }
                        } else {
                            let target = self.find_nearest(world_tiles, peep_pos, |tile| {
                                tile.kind != TileType::City && tile.check_inventory(Objects::Logs)
                            });
                            let travel_to = calculate_movement(peep_pos, target);
                            peep.position = travel_to;
                        }
                    },
                    Action::Deposit(tile_type) => {
                        if tile.kind == tile_type {
                            let _ = tile.deposit_item(Objects::Logs);
                            peep.holding = None;
                        } else {
                            let target = self.find_nearest(world_tiles, peep_pos, |tile| tile.kind == TileType::City);
                            let travel_to = calculate_movement(peep_pos, target);
                            peep.position = travel_to;
                        }
                    },
                },
                Jobs::Miller => match task {
                    Action::Gather(tile_type) => {
                        if let Some(_logs) = peep.holding {
                            if tile.kind == tile_type {
                                peep.holding = Some(Objects::Planks); // was holding logs, now holding planks
                            } else {
                                let target = self.find_nearest(world_tiles, peep_pos, |tile| tile.kind == TileType::City);
                                let travel_to = calculate_movement(peep_pos, target);
                                peep.position = travel_to;
                            }
                        } else if tile.take_item(Objects::Logs, tile_type) {
                            peep.holding = Some(Objects::Logs);
                        } else {
                            let target = self.find_nearest(world_tiles, peep_pos, |tile| {
                                tile.kind != TileType::Forest && tile.check_inventory(Objects::Logs)
                            });
                            let travel_to = calculate_movement(peep_pos, target);
                            peep.position = travel_to;
                        }
                    },
                    Action::Deposit(tile_type) => {
                        if tile.kind == tile_type {
                            let _ = tile.deposit_item(Objects::Planks);
                            peep.holding = None;
                        } else {
                            let target = self.find_nearest(world_tiles, peep_pos, |tile| tile.kind == TileType::City);
                            let travel_to = calculate_movement(peep_pos, target);
                            peep.position = travel_to;
                        }
                    },
                },
                Jobs::Carpenter => match task {
                    Action::Gather(tile_type) => {
                        if tile.take_item(Objects::Planks, tile_type) {
                            peep.holding = Some(Objects::Planks);
                        } else {
                            let target = self.find_nearest(world_tiles, peep_pos, |tile| {
                                tile.kind != TileType::Plains && tile.check_inventory(Objects::Planks)
                            });
                            let travel_to = calculate_movement(peep_pos, target);
                            peep.position = travel_to;
                        }
                    },
                    Action::Deposit(tile_type) => {
                        if tile.kind == tile_type {
                            let planks_at_future_site = tile.deposit_item(Objects::Planks);
                            peep.holding = None;

                            if planks_at_future_site > 200 {
                                tile.kind = TileType::City;
                            }
                        } else {
                            let target = self.find_nearest(world_tiles, peep_pos, |tile| tile.kind == TileType::Plains);
                            let travel_to = calculate_movement(peep_pos, target);
                            peep.position = travel_to;
                        }
                    },
                },
            }
        }

        // Baby making
        // TODO: Put population of each tile into Tile struct to facilitate this? Must consider simulation loop for peepl, 
        //       would we loop through each time and sim the population tile by tile, or have a separate list of all 
        //       population to loop directly through them all?

        // Get city tiles in world
        let cities = {
            let mut c = Vec::new();
            for tile in world_tiles.iter() {
                if tile.kind == TileType::City {
                    c.push(tile.position);
                }
            }
            c
        };

        // Count populations in cities
        let mut new_population: Vec<Peepl> = Vec::new();
        let mut pairs = 0;
        {
            let world_pops = pop_guard.as_slice();
            let mut pop_counts: HashMap<(i64, i64), u64> = HashMap::new();
            for peep in world_pops.iter() {
                if cities.contains(&peep.position) {
                    if let Some(city_pop) = pop_counts.get_mut(&peep.position) {
                        *city_pop += 1;
                    } else {
                        pop_counts.insert(peep.position, 1);
                    }
                }
            }

            for ((x, y), city_pop) in pop_counts.into_iter() {
                let pairs_of_peepl = city_pop / 2;
                pairs = pairs_of_peepl;
                if pairs_of_peepl >= 1 {
                    let mut roll: f64;
                    let chance: u64 = 2;
                    for _ in 0..pairs_of_peepl {
                        roll = rand::random::<f64>() * 1000.0;
                        if chance >= roll as u64 {
                            new_population.push(World::birth_peep((x, y), &self.job_probabilities))
                        }
                    }
                }
            }
        }

        println!("there were {} pairs of peepl in {} cities", pairs, cities.len());
        println!("{} newborns during sim step {}", new_population.len(), self.steps);
        pop_guard.append(&mut new_population);

        println!("Simulation step {} took... {:?}", self.steps, stopwatch.elapsed().unwrap());
        self.steps += 1;
    }

    /// x and y must be >= 0
    fn get_tile_index(&self, x: i64, y: i64) -> usize {
        assert!(x >= 0 && y >= 0, "Coordinates must be positive to calculate index.");
        (y * self.world_size.1 as i64 + x) as usize
    }

    fn find_nearest<F>(&self, tiles: &[Tile], from_pos: (i64, i64), check: F) -> (i64, i64)
        where F: Fn(&Tile) -> bool  {
            let mut checked: HashSet<(i64, i64)> = HashSet::with_capacity(self.world_size.0 * self.world_size.1); // don't like this lazy way, but I couldn't figure it out, just wanted something that worked
            for distance in 0..self.world_size.0 as i64 {
                for off_y in -distance..distance+1 {
                    let cursor_y = from_pos.1 + off_y;
                    if cursor_y < 0 || cursor_y >= self.world_size.1 as i64 { continue; }
                    for off_x in -distance..distance+1 {
                        let cursor_x = from_pos.0 + off_x;
                        if cursor_x < 0 || cursor_x >= self.world_size.0 as i64 { continue; }
                        if checked.contains(&(off_x, off_y)) { continue; } // yeah...
                        else { checked.insert((off_x, off_y)); }

                        let cursor_tile = &tiles[self.get_tile_index(cursor_x, cursor_y)];
                        if check(cursor_tile) {
                            return (cursor_x, cursor_y);
                        }
                    }
                }
            }
            from_pos
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Jobs {
    Woodcutter,
    Miller,
    Carpenter,
}

enum Action {
    Gather(TileType),
    Deposit(TileType),
}

#[derive(Debug)]
struct Peepl {
    position: (i64, i64),
    holding: Option<Objects>,
    job: Jobs,
}

impl Peepl {
    fn step(&self) -> Action {
        match self.job {
            Jobs::Woodcutter => {
                if self.holding.is_some() {
                    Action::Deposit(TileType::City)
                } else {
                    Action::Gather(TileType::Forest)
                }
            },
            Jobs::Miller => {
                if let Some(good) = &self.holding {
                    match good {
                        Objects::Logs => Action::Gather(TileType::City),
                        Objects::Planks => Action::Deposit(TileType::City),
                    }
                } else {
                    Action::Gather(TileType::City)
                }
            },
            Jobs::Carpenter => {
                if self.holding.is_some() {
                    Action::Deposit(TileType::Plains)
                } else {
                    Action::Gather(TileType::City)
                }
            },
        }
    }
}