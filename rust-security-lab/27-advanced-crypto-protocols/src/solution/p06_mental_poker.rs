//! # Lesson 06: Mental Poker (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card(pub u8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckState {
    pub cards: Vec<Card>,
    pub permutation_hashes: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permutation {
    pub mapping: Vec<usize>,
}

/// Create a standard 52-card deck.
pub fn create_standard_deck() -> Vec<Card> {
    (0..52).map(Card).collect()
}

/// Validate that a mapping is a valid permutation.
pub fn validate_permutation(permutation: &Permutation, n: usize) -> Result<(), String> {
    if permutation.mapping.len() != n {
        return Err(format!(
            "Permutation length {} != expected {}",
            permutation.mapping.len(),
            n
        ));
    }

    let mut seen = vec![false; n];
    for &val in &permutation.mapping {
        if val >= n {
            return Err(format!("Value {} out of range [0, {})", val, n));
        }
        if seen[val] {
            return Err(format!("Duplicate value {}", val));
        }
        seen[val] = true;
    }

    Ok(())
}

/// Apply a permutation to a deck.
pub fn apply_permutation(deck: &[Card], permutation: &Permutation) -> Vec<Card> {
    let mut result = vec![Card(0); deck.len()];
    for (i, &pos) in permutation.mapping.iter().enumerate() {
        result[pos] = deck[i];
    }
    result
}

/// Commit to a permutation using SHA-256.
pub fn commit_permutation(permutation: &Permutation) -> (Vec<u8>, Vec<u8>) {
    let mut rng = rand::thread_rng();
    let salt: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();

    let mut data = serde_json::to_vec(permutation).unwrap();
    data.extend_from_slice(&salt);

    let hash = hash_data(&data);
    (hash, salt)
}

/// Verify a permutation commitment.
pub fn verify_commitment(
    permutation: &Permutation,
    salt: &[u8],
    expected_hash: &[u8],
) -> bool {
    let mut data = serde_json::to_vec(permutation).unwrap();
    data.extend_from_slice(salt);
    hash_data(&data) == expected_hash
}

/// Run multi-player shuffle protocol.
pub fn shuffle_deck(num_players: usize) -> (DeckState, Vec<Permutation>) {
    let mut rng = rand::thread_rng();
    let deck_size = 52;

    let mut permutations = Vec::new();
    let mut commitments = Vec::new();

    // Each player creates and commits to a permutation
    for _ in 0..num_players {
        let perm = generate_random_permutation(deck_size, &mut rng);
        let (hash, _salt) = commit_permutation(&perm);
        commitments.push(hash);
        permutations.push(perm);
    }

    // Apply all permutations sequentially
    let mut current_deck = create_standard_deck();
    for perm in &permutations {
        current_deck = apply_permutation(&current_deck, perm);
    }

    (
        DeckState {
            cards: current_deck,
            permutation_hashes: commitments,
        },
        permutations,
    )
}

/// Generate a random permutation of size n using Fisher-Yates shuffle.
fn generate_random_permutation(n: usize, rng: &mut impl Rng) -> Permutation {
    let mut mapping: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = rng.gen_range(0..=i);
        mapping.swap(i, j);
    }
    Permutation { mapping }
}

/// Deal cards to players from a shuffled deck.
pub fn deal_cards(
    deck: &[Card],
    num_players: usize,
    cards_per_player: usize,
) -> (Vec<Vec<Card>>, Vec<Card>) {
    let total_dealt = num_players * cards_per_player;
    let mut hands: Vec<Vec<Card>> = vec![Vec::new(); num_players];

    for (i, card) in deck.iter().take(total_dealt).enumerate() {
        hands[i % num_players].push(*card);
    }

    let draw_pile = deck[total_dealt..].to_vec();
    (hands, draw_pile)
}

pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_standard_deck() {
        let deck = create_standard_deck();
        assert_eq!(deck.len(), 52);
        let mut sorted: Vec<u8> = deck.iter().map(|c| c.0).collect();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 52);
    }

    #[test]
    fn test_validate_permutation_valid() {
        let perm = Permutation {
            mapping: vec![2, 0, 1, 4, 3],
        };
        assert!(validate_permutation(&perm, 5).is_ok());
    }

    #[test]
    fn test_validate_permutation_invalid_duplicate() {
        let perm = Permutation {
            mapping: vec![0, 0, 1],
        };
        assert!(validate_permutation(&perm, 3).is_err());
    }

    #[test]
    fn test_validate_permutation_invalid_range() {
        let perm = Permutation {
            mapping: vec![0, 5, 2],
        };
        assert!(validate_permutation(&perm, 3).is_err());
    }

    #[test]
    fn test_apply_permutation() {
        let deck = vec![Card(0), Card(1), Card(2), Card(3)];
        let perm = Permutation {
            mapping: vec![3, 2, 1, 0],
        };
        let shuffled = apply_permutation(&deck, &perm);
        assert_eq!(shuffled, vec![Card(3), Card(2), Card(1), Card(0)]);
    }

    #[test]
    fn test_apply_identity_permutation() {
        let deck = vec![Card(0), Card(1), Card(2)];
        let perm = Permutation {
            mapping: vec![0, 1, 2],
        };
        let result = apply_permutation(&deck, &perm);
        assert_eq!(result, deck);
    }

    #[test]
    fn test_commit_and_verify() {
        let perm = Permutation {
            mapping: vec![2, 0, 1, 4, 3],
        };
        let (hash, salt) = commit_permutation(&perm);
        assert!(verify_commitment(&perm, &salt, &hash));
    }

    #[test]
    fn test_shuffle_deck_preserves_all_cards() {
        let (state, _perms) = shuffle_deck(3);
        let mut cards: Vec<u8> = state.cards.iter().map(|c| c.0).collect();
        cards.sort();
        cards.dedup();
        assert_eq!(cards.len(), 52);
    }

    #[test]
    fn test_shuffle_deck_committed() {
        let (state, _) = shuffle_deck(3);
        assert_eq!(state.permutation_hashes.len(), 3);
    }

    #[test]
    fn test_deal_cards() {
        let deck = create_standard_deck();
        let (hands, draw_pile) = deal_cards(&deck, 4, 5);
        assert_eq!(hands.len(), 4);
        for hand in &hands {
            assert_eq!(hand.len(), 5);
        }
        assert_eq!(draw_pile.len(), 52 - 20);
    }

    #[test]
    fn test_deal_cards_no_overlap() {
        let deck = create_standard_deck();
        let (hands, draw_pile) = deal_cards(&deck, 2, 5);
        let mut all_cards: Vec<u8> = hands
            .iter()
            .flat_map(|h| h.iter().map(|c| c.0))
            .chain(draw_pile.iter().map(|c| c.0))
            .collect();
        all_cards.sort();
        all_cards.dedup();
        assert_eq!(all_cards.len(), 52);
    }
}
