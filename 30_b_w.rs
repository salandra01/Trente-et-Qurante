use std::collections::{BTreeMap, HashMap};

/// Pack counts (10 ranks) into a compact u64 key.
/// Each count is 0..=15 (we only need 0..=4 here), we use 4 bits per rank.
fn pack_counts(counts: &[u8; 10]) -> u64 {
    let mut key: u64 = 0;
    for (i, &c) in counts.iter().enumerate() {
        key |= (c as u64) << (4 * i);
    }
    key
}

fn unpack_counts(mut key: u64) -> [u8; 10] {
    let mut counts = [0u8; 10];
    for i in 0..10 {
        counts[i] = (key & 0xF) as u8;
        key >>= 4;
    }
    counts
}

/// State key for memoization: (packed_counts, current_total, run_length)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct StateKey {
    counts_key: u64,
    total: u16,
    run_len: u8,
}

/// A joint distribution of terminal outcomes: total → run length → probability
type Dist = BTreeMap<u32, BTreeMap<u32, f64>>;

/// The recursive probability computation with memoization
fn dist_from_state(
    counts_key: u64,
    total: u32,
    run_len: u32,
    target_sum: u32,
    memo: &mut HashMap<StateKey, Dist>,
) -> Dist {
    let key = StateKey {
        counts_key,
        total: total as u16,
        run_len: run_len as u8,
    };
    if let Some(cached) = memo.get(&key) {
        return cached.clone();
    }

    let counts = unpack_counts(counts_key);
    let remaining_cards: u32 = counts.iter().map(|&c| c as u32).sum();

    // Terminal condition: stop if total >= target or no cards remain
    if total >= target_sum || remaining_cards == 0 {
        let mut d = Dist::new();
        d.entry(total)
            .or_insert_with(BTreeMap::new)
            .insert(run_len, 1.0);
        memo.insert(key, d.clone());
        return d;
    }

    let mut result: Dist = Dist::new();

    for (rank_index, &count_u8) in counts.iter().enumerate() {
        let count = count_u8 as u32;
        if count == 0 {
            continue;
        }
        let rank_value = (rank_index as u32) + 1;
        let mut next_counts = counts;
        next_counts[rank_index] -= 1;
        let next_counts_key = pack_counts(&next_counts);

        let p = (count as f64) / (remaining_cards as f64);

        let sub_dist =
            dist_from_state(next_counts_key, total + rank_value, run_len + 1, target_sum, memo);

        // accumulate with weight p
        for (t, sub_map) in sub_dist {
            for (len, subp) in sub_map {
                *result.entry(t).or_insert_with(BTreeMap::new).entry(len).or_insert(0.0) +=
                    p * subp;
            }
        }
    }

    memo.insert(key, result.clone());
    result
}

fn main() {
    // 40-card deck: 4 of each rank 1..10
    let initial_counts: [u8; 10] = [4u8; 10];
    let target_sum: u32 = 31;

    let counts_key = pack_counts(&initial_counts);
    let mut memo: HashMap<StateKey, Dist> = HashMap::new();

    let dist = dist_from_state(counts_key, 0, 0, target_sum, &mut memo);

    println!("Simplified single-line drawing (40-card deck, no face cards)");
    println!("Stop when total >= {}", target_sum);
    println!("Number of reachable memo states: {}", memo.len());
    println!();

    // Flatten into margipnal distributions
    let mut total_dist: BTreeMap<u32, f64> = BTreeMap::new();
    let mut runlen_dist: BTreeMap<u32, f64> = BTreeMap::new();
    let mut expected_total = 0.0;
    let mut expected_runlen = 0.0;

    for (t, sub_map) in &dist {
        for (len, p) in sub_map {
            *total_dist.entry(*t).or_insert(0.0) += p;
            *runlen_dist.entry(*len).or_insert(0.0) += ;
            expected_total += (*t as f64) * p;
            expected_runlen += (*len as f64) * p;
        }
    }

    println!("Terminal total distribution:");
    for (t, p) in &total_dist {
        println!("{:3} -> {:.12}", t, p);
    }
    println!("\nExpected terminal total = {:.12}", expected_total);

    println!("\nRun length distribution:");
    for (len, p) in &runlen_dist {
        println!("{:3} cards -> {:.12}", len, p);
    }
    println!("\nExpected run length = {:.12}", expected_runlen);
}
