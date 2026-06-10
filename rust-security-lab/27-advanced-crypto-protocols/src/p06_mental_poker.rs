//! # Lesson 06: Mental Poker -- Cards Without a Trusted Dealer
//!
//! ## What is Mental Poker?
//!
//! Mental poker allows players to play card games over a network without any trusted dealer.
//! The challenge: how to shuffle and deal cards so that:
//! 1. No player knows the order of the deck
//! 2. Each player can only see their own cards
//! 3. The deck contains exactly the right cards (no duplicates, no missing)
//! 4. Players can verify the game was fair after it ends
//!
//! ## The Shamir-Rivest Protocol (Simplified)
//!
//! 1. **Representation**: Each card is a number. A standard deck = {0, 1, ..., 51}.
//! 2. **Shuffling**: Each player applies a secret random permutation to the deck.
//!    The composition of all permutations is the final shuffle.
//! 3. **Encryption**: Each player encrypts all cards with their own key.
//!    Card c becomes E_1(E_2(...E_n(c)...)) after all players encrypt.
//! 4. **Dealing**: Players take turns "peeling off" one layer of encryption.
//!    Player i can only decrypt layer i, so they only learn the card that ends up in their position.
//!
//! ## Why It Is Hard
//!
//! - **Cheating**: A player might claim a different permutation or use non-bijective mappings
//! - **Verification**: Players must prove they applied valid permutations without revealing them
//! - **Efficiency**: Encrypting/decrypting every card for every round is expensive
//!
//! ## Attack: Permutation Forgery
//!
//! A malicious player could send a non-permutation (e.g., mapping two positions to the same card),
//! effectively duplicating or removing cards. The protocol must verify that each transformation
//! is a valid permutation.
//!
//! ## Simplification
//!
//! In this exercise we simulate the protocol without real encryption. We use permutations
//! directly and track the deck state. A real implementation would use commutative encryption.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// A card represented by its index (0-51 for a standard deck).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card(pub u8);

/// The state of the deck as seen by the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckState {
    /// Current ordering of cards
    pub cards: Vec<Card>,
    /// Commitments to each player's permutation (for verification)
    pub permutation_hashes: Vec<Vec<u8>>,
}

/// A player's secret permutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permutation {
    /// perm[i] = j means position i maps to position j
    pub mapping: Vec<usize>,
}

/// Exercise 1: Create a standard deck of cards.
///
/// A standard deck has 52 cards: 0-51.
/// Cards 0-12 = Spades (A, 2, ..., K)
/// Cards 13-25 = Hearts
/// Cards 26-38 = Diamonds
/// Cards 39-51 = Clubs
pub fn create_standard_deck() -> Vec<Card> {
    todo!("Create a standard 52-card deck")
}

/// Exercise 2: Validate that a mapping is a valid permutation.
///
/// A valid permutation of size n is a bijection from {0, ..., n-1} to itself.
/// Check that:
/// - mapping.len() == n
/// - Every value 0..n-1 appears exactly once
pub fn validate_permutation(permutation: &Permutation, n: usize) -> Result<(), String> {
    todo!("Check that the mapping is a valid permutation")
}

/// Exercise 3: Apply a permutation to a deck.
///
/// new_deck[perm.mapping[i]] = old_deck[i]
/// Or equivalently: new_deck[i] = old_deck[perm.mapping[i]] (depending on convention)
///
/// Use: new_deck[perm.mapping[i]] = old_deck[i] (forward application)
pub fn apply_permutation(deck: &[Card], permutation: &Permutation) -> Vec<Card> {
    todo!("Rearrange deck according to permutation")
}

/// Exercise 4: Commit to a permutation using a hash.
///
/// To prevent cheating, each player commits to their permutation before the shuffle.
/// Commitment = SHA-256(permutation_bytes || salt)
///
/// Return the hash and the salt (salt is revealed later for verification).
pub fn commit_permutation(permutation: &Permutation) -> (Vec<u8>, Vec<u8>) {
    todo!("Create a commitment to a permutation")
}

/// Exercise 5: Verify a permutation commitment.
///
/// Given the permutation, salt, and expected hash, verify the commitment.
pub fn verify_commitment(
    permutation: &Permutation,
    salt: &[u8],
    expected_hash: &[u8],
) -> bool {
    todo!("Verify that a permutation matches its commitment")
}

/// Exercise 6: Run a full shuffle protocol.
///
/// Simulate multiple players each applying their secret permutation:
/// 1. Each player creates a random permutation
/// 2. Each player commits to their permutation
/// 3. Starting from the standard deck, each player applies their permutation in sequence
/// 4. Return the final shuffled deck and all commitments
///
/// The final deck order is determined by all permutations combined.
pub fn shuffle_deck(num_players: usize) -> (DeckState, Vec<Permutation>) {
    todo!("Run multi-player shuffle protocol")
}

/// Exercise 7: Deal cards to players from a shuffled deck.
///
/// Deal `cards_per_player` cards to each player. Return a Vec of Vec<Card>,
/// where dealt_cards[i] = player i's hand.
///
/// After dealing, the remaining cards form the draw pile.
pub fn deal_cards(
    deck: &[Card],
    num_players: usize,
    cards_per_player: usize,
) -> (Vec<Vec<Card>>, Vec<Card>) {
    todo!("Deal cards from the shuffled deck")
}

/// Helper: hash data with SHA-256.
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
        // All cards should be unique
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
            mapping: vec![0, 0, 1], // 0 appears twice
        };
        assert!(validate_permutation(&perm, 3).is_err());
    }

    #[test]
    fn test_validate_permutation_invalid_range() {
        let perm = Permutation {
            mapping: vec![0, 5, 2], // 5 is out of range for size 3
        };
        assert!(validate_permutation(&perm, 3).is_err());
    }

    #[test]
    fn test_apply_permutation() {
        let deck = vec![Card(0), Card(1), Card(2), Card(3)];
        let perm = Permutation {
            mapping: vec![3, 2, 1, 0], // reverse
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

        // Wrong salt should fail
        let wrong_salt = vec![0u8; 16];
        assert!(!verify_permutation_with_wrong_salt(&perm, &wrong_salt, &hash));
    }

    fn verify_permutation_with_wrong_salt(
        perm: &Permutation,
        wrong_salt: &[u8],
        expected_hash: &[u8],
    ) -> bool {
        let mut data = serde_json::to_vec(perm).unwrap();
        data.extend_from_slice(wrong_salt);
        hash_data(&data) == expected_hash
    }

    #[test]
    fn test_shuffle_deck_preserves_all_cards() {
        let (state, perms) = shuffle_deck(3);
        // All 52 cards should still be present
        let mut cards: Vec<u8> = state.cards.iter().map(|c| c.0).collect();
        cards.sort();
        cards.dedup();
        assert_eq!(cards.len(), 52);
    }

    #[test]
    fn test_shuffle_deck_committed() {
        let (state, _) = shuffle_deck(3);
        // Each player should have committed
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
