use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Holds the counts of all observed outcomes from the simulation.
struct SimResults {
    score_counts: HashMap<u8, u64>,
    length_counts: HashMap<u8, u64>,
    total_games: u64,
}

impl SimResults {
    fn new() -> Self {
        SimResults {
            score_counts: HashMap::new(),
            length_counts: HashMap::new(),
            total_games: 0,
        }
    }
}

/// Plays one full game with a shuffled deck and returns the outcome.
/// Returns a tuple of (final_score, game_length).
fn play_game(deck: &mut Vec<u8>) -> (u8, u8) {
    deck.shuffle(&mut thread_rng());

    let mut sum = 0;
    let mut cards_drawn = 0;

    for card in deck.iter() {
        sum += *card;
        cards_drawn += 1;
        if sum > 30 {
            break;
        }
    }
    (sum, cards_drawn)
}

/// Calculates probabilities and saves them to a file and prints to console.
fn report_and_save_results(results: &SimResults) {
    println!("\n--- Simulation Interrupted ---");
    println!("Calculating results from {} total games played.", results.total_games);
    
    if results.total_games == 0 {
        println!("No games were played. Exiting.");
        return;
    }

    // Calculate average score and length
    let total_score_sum: u64 = results
        .score_counts
        .iter()
        .map(|(&score, &count)| score as u64 * count)
        .sum();
    let avg_score = total_score_sum as f64 / results.total_games as f64;

    let total_length_sum: u64 = results
        .length_counts
        .iter()
        .map(|(&length, &count)| length as u64 * count)
        .sum();
    let avg_length = total_length_sum as f64 / results.total_games as f64;

    // Prepare the output string
    let mut output = String::new();
    output.push_str(&format!("Monte Carlo Simulation Results\n"));
    output.push_str(&format!("Total Games Simulated: {}\n\n", results.total_games));

    output.push_str("--- Averages ---\n");
    output.push_str(&format!("Average Score:  {:.4}\n", avg_score));
    output.push_str(&format!("Average Length: {:.4} cards\n\n", avg_length));

    // Score Distribution
    output.push_str("--- Score Distribution ---\n");
    let mut sorted_scores: Vec<_> = results.score_counts.iter().collect();
    sorted_scores.sort_by_key(|&(&score, _)| score);
    for (score, count) in sorted_scores {
        let prob = (*count as f64 / results.total_games as f64) * 100.0;
        output.push_str(&format!("Score: {} | Probability: {:>9.6}%\n", score, prob));
    }

    // Length Distribution
    output.push_str("\n--- Length Distribution ---\n");
    let mut sorted_lengths: Vec<_> = results.length_counts.iter().collect();
    sorted_lengths.sort_by_key(|&(&len, _)| len);
    for (len, count) in sorted_lengths {
        let prob = (*count as f64 / results.total_games as f64) * 100.0;
        output.push_str(&format!("Length: {} | Probability: {:>9.6}%\n", len, prob));
    }

    // Print to console
    println!("{}", output);

    // Save to file
    match File::create("monte_carlo_results.txt") {
        Ok(mut file) => {
            if let Err(e) = file.write_all(output.as_bytes()) {
                eprintln!("Error writing to file: {}", e);
            } else {
                println!("\nResults successfully saved to 'monte_carlo_results.txt'");
            }
        }
        Err(e) => eprintln!("Error creating file: {}", e),
    }
}

fn main() {
    // Create the shared state for results, protected by Arc and Mutex.
    // Arc allows multiple owners, Mutex ensures only one can write at a time.
    let results_data = Arc::new(Mutex::new(SimResults::new()));
    
    // Clone the Arc for the Ctrl+C handler. This increases the reference count.
    let handler_data = Arc::clone(&results_data);

    // Set up the Ctrl+C handler.
    // When Ctrl+C is pressed, this closure will be executed.
    ctrlc::set_handler(move || {
        // Lock the data to get safe access to the results.
        let results = handler_data.lock().unwrap();
        report_and_save_results(&results);
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    println!("Starting simulation... Press Ctrl+C to stop and save results.");

    let mut deck: Vec<u8> = Vec::new();
    for value in 1..=7 {
        for _ in 0..4 {
            deck.push(value);
        }
    } 
    for _ in 0..12 {
        deck.push(10);
    }
    let start_time = Instant::now();

    // The main simulation loop. This will run forever until interrupted.
    loop {
        let (final_score, game_length) = play_game(&mut deck);

        // Lock the data to update the counts. The lock is released automatically
        // when `results` goes out of scope at the end of the block.
        {
            let mut results = results_data.lock().unwrap();
            results.total_games += 1;
            *results.score_counts.entry(final_score).or_insert(0) += 1;
            *results.length_counts.entry(game_length).or_insert(0) += 1;

            // Provide periodic updates to the user without slowing down too much.
            if results.total_games % 1_000_000 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let games_per_sec = results.total_games as f64 / elapsed;
                println!(
                    "Games played: {:>10} ({:.2} million games/sec)",
                    results.total_games,
                    games_per_sec / 1_000_000.0
                );
            }
        }
    }
}

