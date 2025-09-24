use std::collections::HashMap;
use std::time::Instant;

// The Memo key is (current_sum, deck_counts).
// The value is a map of outcomes: {(final_score, cards_drawn_from_this_point) -> probability}
type DeckCounts = [u8; 10];
type Memo = HashMap<(u8, DeckCounts), HashMap<(u8, u8), f64>>;

/// Corrected recursive solver.
fn solve(
    sum: u8,
    counts: DeckCounts,
    memo: &mut Memo,
) -> HashMap<(u8, u8), f64> {
    if sum > 30 {
        return HashMap::from([((sum, 0), 1.0)]);
    }

    if let Some(memoized_result) = memo.get(&(sum, counts)) {
        return memoized_result.clone();
    }

    let total_cards_remaining = counts.iter().sum::<u8>();
    if total_cards_remaining == 0 {
        return HashMap::new();
    }

    let mut all_outcomes: HashMap<(u8, u8), f64> = HashMap::new();

    for i in 0..10 {
        if counts[i] > 0 {
            let card_value = (i + 1) as u8;
            let prob_of_drawing_card = counts[i] as f64 / total_cards_remaining as f64;

            let mut next_counts = counts;
            next_counts[i] -= 1;

            let sub_outcomes = solve(sum + card_value, next_counts, memo);

            for ((final_score, cards_to_finish), sub_prob) in sub_outcomes.iter() {
                let total_prob = prob_of_drawing_card * sub_prob;
                *all_outcomes
                    .entry((*final_score, cards_to_finish + 1))
                    .or_insert(0.0) += total_prob;
            }
        }
    }

    memo.insert((sum, counts), all_outcomes.clone());
    all_outcomes
}

fn main() {
    let start_time = Instant::now();
    let mut memo: Memo = HashMap::new();
    let mut initial_deck: DeckCounts = [0; 10];

    for i in 0..7 {
        initial_deck[i] = 4
    }
    initial_deck[9] = 12;

    let results = solve(0, initial_deck, &mut memo);

    let mut score_probs: HashMap<u8, f64> = HashMap::new();
    let mut length_probs: HashMap<u8, f64> = HashMap::new();

    for ((score, length), prob) in results.iter() {
        *score_probs.entry(*score).or_insert(0.0) += prob;
        *length_probs.entry(*length).or_insert(0.0) += prob;
    }
    
    // --- Results Display ---
    println!("--- Score Distribution (Corrected) ---");
    let mut sorted_scores: Vec<_> = score_probs.into_iter().collect();
    sorted_scores.sort_by_key(|&(score, _)| score);
    let mut total_prob_score = 0.0;
    let mut expected_score = 0.0;
    for (score, prob) in sorted_scores {
        println!("Score: {} | Probability: {:>9.6}%", score, prob * 100.0);
        total_prob_score += prob;
        expected_score += score as f64 * prob;
    }
    println!("--------------------------------------");
    println!("Total Probability: {:.6}%", total_prob_score * 100.0);
    println!("Average Final Score: {:.6}", expected_score);

    println!("\n--- Length Distribution (Corrected) ---");
    let mut sorted_lengths: Vec<_> = length_probs.into_iter().collect();
    sorted_lengths.sort_by_key(|&(len, _)| len);
    let mut total_prob_length = 0.0;
    let mut expected_length = 0.0;
    for (length, prob) in sorted_lengths {
        println!("Length: {} | Probability: {:>9.6}%", length, prob * 100.0);
        total_prob_length += prob;
        expected_length += length as f64 * prob;
    }
    println!("---------------------------------------");
    println!("Total Probability: {:.6}%", total_prob_length * 100.0);
    println!("Average Run Length: {:.6}", expected_length);

    println!("\nCalculation finished in {:?}", start_time.elapsed());
}
