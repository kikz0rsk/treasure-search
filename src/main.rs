use std::io::Write;
use std::cell;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use crate::core::Chromosome;

mod core;

type INSTR = u8;

fn random_vector(rng: &mut Pcg64) -> Vec<INSTR> {
    let mut output: Vec<INSTR> = vec![0; 64];
    for i in 0..16 {
        output[i] = rng.gen_range(0..=u8::MAX);
    }
    return output;
}

fn main() {
    //let mut rng = Pcg64::seed_from_u64(948464);
    let mut rng = Pcg64::from_entropy();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Too few arguments!");
        eprintln!("Arguments: <Number of subjects> <Target generation number> <Mutation probability> <Selection method>");
        eprintln!("Selection methods: 0 - Roulette, 1 - Tournament");
        return;
    }

    let subjects_num = args[1].parse::<u32>().unwrap();
    if subjects_num < 20 {
        eprintln!("Minimum number of subjects is 20!");
        return;
    }
    let mut target_generations = args[2].parse::<u32>().unwrap();
    if target_generations < 1 {
        eprintln!("Minimum number of generations is 1!");
        return;
    }
    let mutation_probability = args[3].parse::<f64>().unwrap();
    let selection_method: u8 = args[4].parse().unwrap();    // 0 - ruleta, 1 - turnaj
    if selection_method > 1 {
        eprintln!("Invalid selection method!");
        return;
    }

    let game_area: Vec<Vec<u8>> = core::build_game_area();
    let mut player_x: isize = 0;
    let mut player_y: isize = 0;
    let mut treasures: u32 = 0;
    for y in 0..game_area.len() {
        for x in 0..game_area[0].len() {
            if game_area[y][x] == core::GAME_AREA_PLAYER {
                player_x = isize::try_from(x).unwrap();
                player_y = isize::try_from(y).unwrap();
                print!("P ");
            } else if game_area[y][x] == core::GAME_AREA_TREASURE {
                treasures += 1;
                print!("█ ");
            } else {
                print!("░ ");
            }
        }
        println!();
    }

    let mut current_generation: Vec<core::Chromosome> = Vec::new();

    for _ in 0..subjects_num {
        let instructions = random_vector(&mut rng);

        current_generation.push(core::Chromosome {
            genes: instructions,
            found_treasures: 0,
            fitness: 0f64,
            iterations: 0,
            steps: String::new(),
        });
    }

    let mut generations: u32 = 0;
    let mut best_so_far: Option<Chromosome> = Option::None;
    loop {
        if generations > target_generations - 1 {
            let best_so_far = best_so_far.as_ref().unwrap();
            println!("\nTarget generation reached!");
            println!("\nBest solution so far: Generation: {}, Fitness: {}, Steps: {} ({}), Treasures: {}, Iterations: {}",
                     generations, best_so_far.fitness, best_so_far.steps, best_so_far.steps.len(), best_so_far.found_treasures, best_so_far.iterations);
            println!("{:?}", best_so_far.genes);
            print!("Do you want to keep searching for a better solution? y/N: ");
            std::io::stdout().flush();
            let mut ans = String::new();
            std::io::stdin().read_line(&mut ans).unwrap();
            if !ans.trim().eq_ignore_ascii_case("y") {
                return;
            }
            target_generations = u32::MAX;
        }
        generations += 1;
        if generations % 500 == 0 {
            print!("\r\t\t\t\t\t\t\t\r");
            if best_so_far.is_some() {
                let best_so_far_ref = best_so_far.as_ref().unwrap();
                print!("Generation {}; F: {:.4}, T: {}, S: {}, I: {}",
                       generations, best_so_far_ref.fitness,
                       best_so_far_ref.found_treasures, best_so_far_ref.steps.len(), best_so_far_ref.iterations);
            }
            std::io::stdout().flush();
        }
        for i in 0..current_generation.len() {
            let mut current_chromosome = current_generation.get_mut(i).unwrap();
            let mut steps: String = String::new();
            let (iters, found_treasures) = core::run_virtual_machine(&current_chromosome.genes, &game_area, &mut steps, player_x, player_y, treasures);
            let mut fitness: f64 = found_treasures as f64 / treasures as f64;
            fitness -= steps.len() as f64 * 0.005;
            if fitness < 0.0 {
                fitness = 0.0;
            }

            current_chromosome.found_treasures = found_treasures;
            current_chromosome.iterations = iters;
            current_chromosome.steps = steps;
            current_chromosome.fitness = fitness;
        }

        current_generation.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        let mut total_fitness: f64 = 0f64;
        for chromosome in &current_generation {
            total_fitness += chromosome.fitness;
            if chromosome.found_treasures == treasures && (best_so_far.is_none() || chromosome.fitness > best_so_far.as_ref().unwrap().fitness) {
                println!("\nSuccessful solution! Generation: {}, Fitness: {}, Steps: {} ({}), Iterations: {}",
                         generations, chromosome.fitness, chromosome.steps, chromosome.steps.len(), chromosome.iterations);
                println!("{:?}", chromosome.genes);
                print!("Do you want to keep searching for a better solution? y/N: ");
                std::io::stdout().flush();
                let mut ans = String::new();
                std::io::stdin().read_line(&mut ans).unwrap();
                if !ans.trim().eq_ignore_ascii_case("y") {
                    return;
                }
            }
        }

        let mut new_generation: Vec<Chromosome> = Vec::new();
        while new_generation.len() < subjects_num as usize {
            let (parent1, parent2) = if selection_method == 0 {
                core::selection_roulette(&current_generation, total_fitness, &mut rng)
            } else {
                core::selection_tournament(&current_generation, &mut rng)
            };

            let mut iterations = subjects_num as usize - new_generation.len();
            if iterations > core::NUM_OF_CHILDREN as usize {
                iterations = core::NUM_OF_CHILDREN as usize;
            }
            for _ in 0..iterations {
                new_generation.push(core::Chromosome {
                    genes: core::reproduce(parent1, parent2, mutation_probability, &mut rng),
                    fitness: 0.0,
                    steps: String::new(),
                    found_treasures: 0,
                    iterations: 0,
                })
            }
        }

        assert_eq!(new_generation.len(), subjects_num as usize);
        let local_best: Option<Chromosome> = Option::Some(current_generation.swap_remove(0));
        if best_so_far.is_none() {
            best_so_far = local_best;
        } else if local_best.as_ref().unwrap().fitness > best_so_far.as_ref().unwrap().fitness {
            best_so_far = local_best;
        }
        current_generation = new_generation;
    }
}