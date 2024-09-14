use std::{collections::{HashMap, HashSet}, fmt::Display, io, time::SystemTime};

fn main() {
    let mut jerbs: Vec<(Jobs, usize)> = Vec::new();
    jerbs.push((Jobs::Carpenter, 1));
    jerbs.push((Jobs::Miller, 1));
    jerbs.push((Jobs::Woodcutter, 2));

    let mut tiles: Vec<(TileType, usize)> = Vec::new();
    tiles.push((TileType::City, 1));
    tiles.push((TileType::Forest, 6));
    tiles.push((TileType::Plains, 3));

    let world_id = World::generate_world(8, 5, jerbs, tiles);

    let world = unsafe { WORLDS.get_mut(world_id).unwrap() };

    let mut exit = false;
    let mut input_buffer = String::new();
    while !exit {
        println!("\nsim step {}\n{}", world.steps, world);
        world.step_simulation();
        io::stdin().read_line(&mut input_buffer).unwrap();
        if !input_buffer.to_lowercase().contains("q") {
            input_buffer = "".to_string();
        } else {
            exit = true;
        }
    }
}

static mut WORLDS: Vec<World> = Vec::new();
static mut POPULATIONS: Vec<WorldPopulation> = Vec::new();
static mut TILES: Vec<WorldTiles> = Vec::new();

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

impl Into<Tile> for TileType {
    fn into(self) -> Tile {
        Tile {
            kind: self,
            position: (0, 0),
            holding: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct Tile {
    kind: TileType,
    position: (usize, usize), // TODO: Position being usize is not worth the trouble of constantly casting, should be i64 or i32
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

struct WorldPopulation {
    world_id: usize,
    pop: Vec<Peepl>,
}

struct WorldTiles {
    world_id: usize,
    size: (usize, usize),
    tiles: Vec<Tile>,
}

impl WorldTiles {
    fn get_tile_index(&self, x: usize, y: usize) -> usize {
        y * self.size.1 + x
    }
    
    fn get_tile_at_pos(&self, x: usize, y: usize) -> &Tile {
        let index = self.get_tile_index(x, y);
        &self.tiles[index]
    }
    
    fn get_mut_tile_at_pos(&mut self, x: usize, y: usize) -> &mut Tile {
        let index = self.get_tile_index(x, y);
        &mut self.tiles[index]
    }

    fn find_nearest_item(&self, from_pos: (usize, usize), looking_for: Objects, ignore_tile_kind: TileType) -> (usize, usize) {
        let mut checked: HashSet<(i32, i32)> = HashSet::with_capacity(self.size.0 * self.size.1); // don't like this lazy way, but I couldn't figure it out, just wanted something that worked
        for distance in 0..self.size.0 as i32 {
            for off_y in distance*-1..distance+1 {
                let cursor_y = from_pos.1 as i32 + off_y;
                if cursor_y < 0 || cursor_y >= self.size.1 as i32 { continue; }
                for off_x in distance*-1..distance+1 {
                    let cursor_x = from_pos.0 as i32 + off_x;
                    if cursor_x < 0 || cursor_x >= self.size.0 as i32 { continue; }
                    if checked.contains(&(off_x, off_y)) { continue; } // yeah...
                    else { checked.insert((off_x, off_y)); }

                    let cursor_x: usize = cursor_x as usize;
                    let cursor_y: usize = cursor_y as usize;
                    let cursor_tile = self.get_tile_at_pos(cursor_x, cursor_y);
                    if cursor_tile.kind == ignore_tile_kind { continue; }
                    else if cursor_tile.check_inventory(looking_for) {
                        return (cursor_x, cursor_y);
                    }
                }
            }
        }

        from_pos
    }

    fn find_nearest_tile(&self, from_pos: (usize, usize), looking_for: TileType) -> (usize, usize) {
        let mut checked: HashSet<(i32, i32)> = HashSet::with_capacity(self.size.0 * self.size.1); // don't like this lazy way, but I couldn't figure it out, just wanted something that worked
        for distance in 0..self.size.0 as i32 {
            for off_y in distance*-1..distance+1 {
                let cursor_y = from_pos.1 as i32 + off_y;
                if cursor_y < 0 || cursor_y >= self.size.1 as i32 { continue; }
                for off_x in distance*-1..distance+1 {
                    let cursor_x = from_pos.0 as i32 + off_x;
                    if cursor_x < 0 || cursor_x >= self.size.0 as i32 { continue; }
                    if checked.contains(&(off_x, off_y)) { continue; } // yeah...
                    else { checked.insert((off_x, off_y)); }

                    let cursor_x: usize = cursor_x as usize;
                    let cursor_y: usize = cursor_y as usize;
                    let cursor_tile = self.get_tile_at_pos(cursor_x, cursor_y);
                    if cursor_tile.kind == looking_for { return (cursor_x, cursor_y); }
                }
            }
        }

        from_pos
    }
}

fn calculate_movement(from: (usize, usize), to: (usize, usize)) -> (usize, usize) {
    let x_offset = if from.0 > to.0 { -1 }
                        else if from.0 == to.0 { 0 }
                        else { 1 };
    let y_offset = if from.1 > to.1 { -1 }
                        else if from.1 == to.1 { 0 }
                        else { 1 };
    let x: usize = (from.0 as i32 + x_offset) as usize;
    let y: usize = (from.1 as i32 + y_offset) as usize;
    (x, y)
}

fn get_population(index: usize) -> &'static WorldPopulation {
    unsafe { POPULATIONS.get(index).unwrap() }
}

fn get_mut_population(index: usize) -> &'static mut WorldPopulation {
    unsafe { POPULATIONS.get_mut(index).unwrap() }
}

fn get_tiles(index: usize) -> &'static WorldTiles {
    unsafe { TILES.get(index).unwrap() }
}

fn get_mut_tiles(index: usize) -> &'static mut WorldTiles {
    unsafe { TILES.get_mut(index).unwrap() }
}

struct World {
    world_id: usize,
    job_probabilities: Vec<(Jobs, f32, f32)>,
    steps: usize,
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // size data from TILES data
        let tiles = get_tiles(self.world_id);
        let size = tiles.size;
        let width = size.0;
        let height = size.1;

        // population data from POPULATIONS data
        let pop = &get_population(self.world_id).pop;

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
                write!(f, "{}", tiles.get_tile_at_pos(x, y).kind)?;
            }
            writeln!(f)?;
        }
        

        std::fmt::Result::Ok(())
    }
}

impl World {
    fn birth_peep(position: (usize, usize), job_probabilities: &Vec<(Jobs, f32, f32)>) -> Peepl {
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
    fn generate_world(size: usize, starting_population_per_city: usize, jobs: Vec<(Jobs, usize)>, tile_makeup: Vec<(TileType, usize)>) -> usize {
        // Generate world job probability table
        let total_job_probability = jobs.iter().fold(0, |total, (_, val) | total + val);
        let world_jobs = {
            let mut world_jobs: Vec<(Jobs, f32, f32)> = Vec::with_capacity(jobs.len());
            let mut acc: f32 = 0.0;
            for (job, integer_prob) in jobs {
                let lower = acc;
                acc = acc + integer_prob as f32 / total_job_probability as f32;
                world_jobs.push((job, lower, acc));
            }
            world_jobs
        };

        // Generate world tiles
        let total_tile_probability = tile_makeup.iter().fold(0, |total, (_, val) | total + val);
        let tile_probs = {
            let mut tile_probs: Vec<(TileType, f32, f32)> = Vec::with_capacity(tile_makeup.len());
            let mut acc: f32 = 0.0;
            for (tile_kind, integer_prob) in tile_makeup {
                let lower = acc;
                acc = acc + integer_prob as f32 / total_tile_probability as f32;
                tile_probs.push((tile_kind, lower, acc));
            }
            tile_probs
        };

        let mut world_tiles: Vec<Tile> = Vec::with_capacity(size*size);
        let mut world_population: Vec<Peepl> = Vec::new();
        for y in 0..size {
            for x in 0..size {
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

        let id = unsafe { WORLDS.len() };
        let world = World { world_id: id, job_probabilities: world_jobs, steps: 0 };
        unsafe {
            WORLDS.push(world);
            POPULATIONS.push(WorldPopulation { world_id: id, pop: world_population });
            TILES.push(WorldTiles { world_id: id, size: (size, size), tiles: world_tiles });
        }

        id
    }

    fn step_simulation(&mut self) {
        let stopwatch = SystemTime::now();

        let world_tiles = get_mut_tiles(self.world_id);

        // Sim all population
        for peep in get_mut_population(self.world_id).pop.iter_mut() {
            let task = peep.step();
            let tile = world_tiles.get_mut_tile_at_pos(peep.position.0, peep.position.1);
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
                            let target = world_tiles.find_nearest_item(peep_pos, Objects::Logs, TileType::City);
                            let travel_to = calculate_movement(peep_pos, target);
                            peep.position = travel_to;
                        }
                    },
                    Action::Deposit(tile_type) => {
                        if tile.kind == tile_type {
                            let _ = tile.deposit_item(Objects::Logs);
                            peep.holding = None;
                        } else {
                            let target = world_tiles.find_nearest_tile(peep_pos, TileType::City);
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
                                let target = world_tiles.find_nearest_tile(peep_pos, TileType::City);
                                let travel_to = calculate_movement(peep_pos, target);
                                peep.position = travel_to;
                            }
                        } else {
                            if tile.take_item(Objects::Logs, tile_type) {
                                peep.holding = Some(Objects::Logs);
                            } else {
                                let target = world_tiles.find_nearest_item(peep_pos, Objects::Logs, TileType::Forest);
                                let travel_to = calculate_movement(peep_pos, target);
                                peep.position = travel_to;
                            }
                        }
                    },
                    Action::Deposit(tile_type) => {
                        if tile.kind == tile_type {
                            let _ = tile.deposit_item(Objects::Planks);
                            peep.holding = None;
                        } else {
                            let target = world_tiles.find_nearest_tile(peep_pos, TileType::City);
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
                            let target = world_tiles.find_nearest_item(peep_pos, Objects::Planks, TileType::Plains);
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
                            let target = world_tiles.find_nearest_tile(peep_pos, TileType::Plains);
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
            for tile in world_tiles.tiles.iter() {
                if tile.kind == TileType::City {
                    c.push(tile.position);
                }
            }
            c
        };

        // Count populations in cities
        let world_pops = get_population(self.world_id);
        let mut pop_counts: HashMap<(usize, usize), u64> = HashMap::new();
        for peep in world_pops.pop.iter() {
            if cities.contains(&peep.position) {
                if let Some(city_pop) = pop_counts.get_mut(&peep.position) {
                    *city_pop += 1;
                } else {
                    pop_counts.insert(peep.position, 1);
                }
            }
        }

        let mut new_population: Vec<Peepl> = Vec::new();
        for ((x, y), city_pop) in pop_counts.into_iter() {
            let pairs_of_peepl = city_pop / 2;
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

        println!("{} newborns during sim step {}", new_population.len(), self.steps);
        let world_pops = get_mut_population(self.world_id);
        for peep in new_population {
            world_pops.pop.push(peep);
        }

        println!("Simulation step {} took... {:?}", self.steps, stopwatch.elapsed().unwrap());
        self.steps += 1;
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
    position: (usize, usize), // TODO: Position being usize is not worth the trouble of constantly casting, should be i64 or i32
    holding: Option<Objects>,
    job: Jobs,
}

impl Peepl {
    fn step(&self) -> Action {
        match self.job {
            Jobs::Woodcutter => {
                if let Some(_) = self.holding {
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
                if let Some(_) = self.holding {
                    Action::Deposit(TileType::Plains)
                } else {
                    Action::Gather(TileType::City)
                }
            },
        }
    }
}