use std::collections::HashMap;
use std::time::Instant;

// The Memo key is (current_sum, deck_counts).
// The value is a map of outcomes: {(final_score, cards_drawn_from_this_point) -> probability}
type DeckCounts = [u16; 10];
type Memo = HashMap<(u16, DeckCounts), HashMap<(u16, u16), f64>>;

/// Corrected recursive solver.
fn solve(
    sum: u16,
    counts: DeckCounts,
    memo: &mut Memo,
) -> HashMap<(u16, u16), f64> {
    // --- Corrected Base Case ---
    // If the sum is over 30, the game is already over.
    // It takes 0 more cards to finish from this point. The probability is 1.0.
    if sum > 30 {
        return HashMap::from([((sum, 0), 1.0)]);
    }

    // Memoization check
    if let Some(memoized_result) = memo.get(&(sum, counts)) {
        return memoized_result.clone();
    }

    let total_cards_remaining = counts.iter().sum::<u16>();
    if total_cards_remaining == 0 {
        return HashMap::new();
    }

    let mut all_outcomes: HashMap<(u16, u16), f64> = HashMap::new();

    for i in 0..10 {
        if counts[i] > 0 {
            let card_value = (i + 1) as u16;
            let prob_of_drawing_card = counts[i] as f64 / total_cards_remaining as f64;

            let mut next_counts = counts;
            next_counts[i] -= 1;

            let sub_outcomes = solve(sum + card_value, next_counts, memo);

            // --- Corrected Recursive Step ---
            for ((final_score, cards_to_finish), sub_prob) in sub_outcomes.iter() {
                let total_prob = prob_of_drawing_card * sub_prob;
                // The key change: We add 1 to the length returned by the sub-problem
                // to account for the card we just drew.
                *all_outcomes
                    .entry((*final_score, cards_to_finish + 1))
                    .or_insert(0.0) += total_prob;
            }
        }
    }

    // Memoize and return
    memo.insert((sum, counts), all_outcomes.clone());
    all_outcomes
}

fn main() {
    let start_time = Instant::now();
    let mut memo: Memo = HashMap::new();
    let mut initial_deck: DeckCounts = [24; 10];
    initial_deck[9] = 96;

    // The initial call to solve(0, ...) will return the final scores and total game lengths.
    let results = solve(0, initial_deck, &mut memo);

    let mut score_probs: HashMap<u16, f64> = HashMap::new();
    let mut length_probs: HashMap<u16, f64> = HashMap::new();

    for ((score, length), prob) in results.iter() {
        *score_probs.entry(*score).or_insert(0.0) += prob;
        *length_probs.entry(*length).or_insert(0.0) += prob;
    }
    
    // --- Results Display ---
    println!("--- Score Distribution (Corrected) ---");
    let mut sorted_scores: Vec<_> = score_probs.into_iter().collect();
    sorted_scores.sort_by_key(|&(score, _)| score);
    let mut total_prob_score = 0.0;
    for (score, prob) in sorted_scores {
        println!("Score: {} | Probability: {:>9.6}%", score, prob * 100.0);
        total_prob_score += prob;
    }
    println!("--------------------------------------");
    println!("Total Probability: {:.6}%", total_prob_score * 100.0);
    
    println!("\n--- Length Distribution (Corrected) ---");
    let mut sorted_lengths: Vec<_> = length_probs.into_iter().collect();
    sorted_lengths.sort_by_key(|&(len, _)| len);
    let mut total_prob_length = 0.0;
    for (length, prob) in sorted_lengths {
        println!("Length: {} | Probability: {:>9.6}%", length, prob * 100.0);
        total_prob_length += prob;
    }
    println!("---------------------------------------");
    println!("Total Probability: {:.6}%", total_prob_length * 100.0);

    println!("\nCalculation finished in {:?}", start_time.elapsed());
}