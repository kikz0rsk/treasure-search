use rand::Rng;
use rand_pcg::Pcg64;

pub const AREA_TILE_PLAYER: u8 = 1;
pub const AREA_TILE_TREASURE: u8 = 2;
pub const AREA_TILE_NOTHING: u8 = 0;

pub const DIR_UP: usize = 0;
pub const DIR_RIGHT: usize = 1;
pub const DIR_DOWN: usize = 2;
pub const DIR_LEFT: usize = 3;

pub const NUM_OF_CHILDREN: i32 = 2;

pub type INSTR = u8;

pub struct Chromosome {
    pub genes: Vec<INSTR>,
    pub found_treasures: u32,
    pub fitness: f64,
    pub iterations: u32,
    pub steps: String,
}

impl Chromosome {
    pub fn with_instructions(instructions: Vec<INSTR>) -> Chromosome {
        return Chromosome {
            genes: instructions,
            found_treasures: 0,
            fitness: 0.0,
            iterations: 0,
            steps: String::new(),
        };
    }
}

pub fn random_instructions(rng: &mut Pcg64) -> Vec<INSTR> {
    let mut output: Vec<INSTR> = vec![0; 64];
    for i in 0..16 {
        output[i] = rng.gen_range(0..=u8::MAX);
    }
    return output;
}

pub fn run_virtual_machine(instructions: &Vec<u8>, original_game_area: &Vec<Vec<u8>>, steps: &mut String, mut player_x: isize, mut player_y: isize, treasures: u32) -> (u32, u32) {
    let rows = original_game_area.len();
    let columns = original_game_area[0].len();

    let mut game_area = original_game_area.clone();
    let mut machine_memory: Vec<u8> = instructions.clone();
    let mut curr_instr_index: usize = 0;
    let mut iterations: u32 = 0;
    let mut found_treasures: u32 = 0;
    while iterations < 500 && curr_instr_index < 64 && found_treasures < treasures {
        let instruction: u8 = machine_memory[curr_instr_index];

        let operation: u8 = instruction & 0xC0;
        let data: usize = usize::from(instruction & 0x3F);
        let mut jump: bool = false;
        match operation {
            0 => {
                // Increment
                machine_memory[data] = cyclic_increment_u8(machine_memory[data]);
            }
            64 => {
                // Decrement
                machine_memory[data] = cyclic_decrement_u8(machine_memory[data]);
            }
            128 => {
                // Jump
                curr_instr_index = data;
                jump = true;
            }
            192 => {
                // Move (print)
                match data & 3 {
                    DIR_UP => {
                        //print!("H");
                        steps.push('H');
                        player_y -= 1;
                    }
                    DIR_RIGHT => {
                        //print!("P");
                        steps.push('P');
                        player_x += 1;
                    }
                    DIR_DOWN => {
                        //print!("D");
                        steps.push('D');
                        player_y += 1;
                    }
                    DIR_LEFT => {
                        //print!("L");
                        steps.push('L');
                        player_x -= 1;
                    }
                    _ => {}
                }
                if !(player_x >= 0 && player_x < (columns as isize) && player_y >= 0 && player_y < (rows as isize)) {
                    break;
                }
                if game_area[player_y as usize][player_x as usize] == AREA_TILE_TREASURE {
                    game_area[player_y as usize][player_x as usize] = 0;
                    found_treasures += 1;
                }
            }
            _ => {}
        }
        iterations += 1;
        if !jump {
            curr_instr_index += 1;
        }
    }
    return (iterations, found_treasures);
}

pub fn reproduce(parent1: &Chromosome, parent2: &Chromosome, mutation_probability: f64, rng: &mut Pcg64) -> Vec<INSTR> {
    let mut output_vector = Vec::new();
    for i in 0..64 {
        let mut mask: u8 = 128;
        let mut number: u8 = 0;
        for _ in 0..8 {
            debug_assert_eq!(number & mask, 0);
            if rng.gen_bool(0.5) {  // Parent 1
                number |= parent1.genes[i] & mask;
            } else {    // Parent 2
                number |= parent2.genes[i] & mask;
            }

            // Mutation
            if rng.gen_bool(mutation_probability) {
                number ^= mask;
            }
            mask >>= 1;
        }
        output_vector.push(number);
    }
    return output_vector;
}

pub fn selection_roulette<'a>(chromosomes: &'a Vec<Chromosome>, total_fitness: f64, rng: &mut Pcg64) -> (&'a Chromosome, &'a Chromosome) {
    let mut v: Vec<&Chromosome> = Vec::with_capacity(2);
    for _ in 0..2 {
        let r: f64 = rng.gen_range(0f64..=total_fitness);
        let mut curr_fitness: f64 = 0f64;
        let mut selected_parent: Option<&Chromosome> = Option::None;
        for c in chromosomes {
            curr_fitness += c.fitness;
            if curr_fitness > r {
                selected_parent = Option::Some(c);
                break;
            }
        }
        if selected_parent.is_none() {
            selected_parent = chromosomes.last();
        }
        v.push(selected_parent.unwrap());
    }
    return (v[0], v[1]);
}

pub fn selection_tournament<'a>(chromosomes: &'a Vec<Chromosome>, rng: &mut Pcg64) -> (&'a Chromosome, &'a Chromosome) {
    let mut v: Vec<&Chromosome> = Vec::with_capacity(2);
    for _ in 0..2 {
        let index1 = rng.gen_range(0..chromosomes.len());
        let index2 = rng.gen_range(0..chromosomes.len());
        if chromosomes[index1].fitness > chromosomes[index2].fitness {
            v.push(&chromosomes[index1]);
        } else {
            v.push(&chromosomes[index2]);
        }
    }
    return (v[0], v[1]);
}

pub fn calculate_fitness(steps: usize, found_treasures: u32, all_treasures: u32) -> f64 {
    let mut fitness: f64 = found_treasures as f64 / all_treasures as f64;
    fitness -= steps as f64 * 0.005;
    if fitness < 0.0 {
        fitness = 0.0;
    }
    return fitness;
}

pub fn build_game_area() -> Vec<Vec<u8>> {
    let mut game_area: Vec<Vec<u8>> = vec![vec![AREA_TILE_NOTHING; 7]; 7];
    game_area[1][4] = AREA_TILE_TREASURE;
    game_area[2][2] = AREA_TILE_TREASURE;
    game_area[3][6] = AREA_TILE_TREASURE;
    game_area[4][1] = AREA_TILE_TREASURE;
    game_area[5][4] = AREA_TILE_TREASURE;
    game_area[6][3] = AREA_TILE_PLAYER;
    return game_area;
}

#[inline]
fn cyclic_increment_u8(n: u8) -> u8 {
    if n == u8::MAX {
        return 0;
    }
    return n + 1;
}

#[inline]
fn cyclic_decrement_u8(n: u8) -> u8 {
    if n == u8::MIN {
        return 0;
    }
    return n - 1;
}
