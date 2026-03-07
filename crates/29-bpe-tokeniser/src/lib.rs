// ============================================================
//  YOUR CHALLENGE — implement Byte-Pair Encoding (BPE) tokenisation.
//
//  BPE is the tokenisation algorithm behind GPT, LLaMA, and Claude.
//  It starts from individual characters, then greedily merges the
//  most frequent adjacent pair until the vocab reaches the target size.
//
//  Algorithm:
//    1. Initialise: each unique char in the corpus is a base token
//    2. Count adjacent pair frequencies across all token sequences
//    3. Merge the most frequent pair into a new token
//    4. Repeat 2-3 until vocab_size == target
//
//  Encoding: apply merge rules in order to turn text into token IDs.
//  Decoding: map IDs back to token strings and concatenate.
//
//  Used in: GPT-{2,3,4}, LLaMA, Whisper, T5, BART, Claude.
// ============================================================

use std::collections::HashMap;

pub struct BpeTokeniser {
    vocab:        HashMap<String, u32>,
    pub merges:   Vec<(String, String)>,
    id_to_token:  Vec<String>,
}

impl BpeTokeniser {
    /// Create an empty tokeniser. Call `train()` before encoding.
    pub fn new() -> Self {
        Self { vocab: HashMap::new(), merges: Vec::new(), id_to_token: Vec::new() }
    }

    /// Learn merge rules from `corpus` until vocab reaches `target_vocab_size`.
    /// Always starts fresh from character-level base tokens.
    pub fn train(&mut self, corpus: &str, target_vocab_size: usize) {
        self.vocab.clear();
        self.merges.clear();
        self.id_to_token.clear();

        // Build character-level base vocabulary from the corpus
        for ch in corpus.chars() {
            let s = ch.to_string();
            if !self.vocab.contains_key(&s) {
                self.vocab.insert(s.clone(), self.id_to_token.len() as u32);
                self.id_to_token.push(s);
            }
        }

        // Represent corpus as token sequences (split on whitespace, char-level)
        let mut token_seqs: Vec<Vec<String>> = corpus
            .split_whitespace()
            .map(|w| w.chars().map(|c| c.to_string()).collect())
            .collect();

        // Greedily merge until target vocab size is reached
        while self.id_to_token.len() < target_vocab_size {
            let freqs = pair_frequencies(&token_seqs);
            if freqs.is_empty() { break; }

            let best = freqs.into_iter().max_by_key(|(_, f)| *f).unwrap().0;
            let merged = format!("{}{}", best.0, best.1);

            // Skip if this merged token already exists in vocab
            if self.vocab.contains_key(&merged) { break; }

            self.vocab.insert(merged.clone(), self.id_to_token.len() as u32);
            self.id_to_token.push(merged.clone());
            self.merges.push(best.clone());

            token_seqs = token_seqs.into_iter()
                .map(|seq| merge_pair(&seq, &best))
                .collect();
        }
    }

    /// Encode `text` by applying learned merge rules, returning token IDs.
    pub fn encode(&self, text: &str) -> Vec<u32> {
        let mut tokens: Vec<String> = text.chars().map(|c| c.to_string()).collect();
        for merge in &self.merges {
            tokens = merge_pair(&tokens, merge);
        }
        tokens.iter()
            .map(|t| self.vocab.get(t).copied().unwrap_or(0))
            .collect()
    }

    /// Decode token IDs back to a string.
    pub fn decode(&self, ids: &[u32]) -> String {
        ids.iter()
            .filter_map(|&id| self.id_to_token.get(id as usize))
            .cloned()
            .collect()
    }

    pub fn vocab_size(&self) -> usize {
        self.id_to_token.len()
    }
}

impl Default for BpeTokeniser {
    fn default() -> Self { Self::new() }
}

/// Count frequencies of all adjacent pairs across all token sequences.
pub fn pair_frequencies(seqs: &[Vec<String>]) -> HashMap<(String, String), usize> {
    let mut freqs: HashMap<(String, String), usize> = HashMap::new();
    for seq in seqs {
        for pair in seq.windows(2) {
            *freqs.entry((pair[0].clone(), pair[1].clone())).or_insert(0) += 1;
        }
    }
    freqs
}

/// Replace all occurrences of `pair` with its merged token in `tokens`.
pub fn merge_pair(tokens: &[String], pair: &(String, String)) -> Vec<String> {
    let mut result = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if i + 1 < tokens.len() && tokens[i] == pair.0 && tokens[i + 1] == pair.1 {
            result.push(format!("{}{}", pair.0, pair.1));
            i += 2;
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    mod helpers {
        use super::*;

        #[test]
        fn pair_frequencies_counts_adjacent_pairs() {
            let seqs = vec![vec!["a".to_string(), "b".to_string(), "a".to_string(), "b".to_string()]];
            let freqs = pair_frequencies(&seqs);
            assert_eq!(freqs[&("a".to_string(), "b".to_string())], 2);
            assert_eq!(freqs[&("b".to_string(), "a".to_string())], 1);
        }

        #[test]
        fn merge_pair_replaces_all_occurrences() {
            let tokens = vec!["a", "b", "c", "a", "b"].iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let pair   = ("a".to_string(), "b".to_string());
            let result = merge_pair(&tokens, &pair);
            assert_eq!(result, vec!["ab", "c", "ab"]);
        }

        #[test]
        fn merge_pair_no_change_when_not_found() {
            let tokens = vec!["x", "y", "z"].iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let pair   = ("a".to_string(), "b".to_string());
            let result = merge_pair(&tokens, &pair);
            assert_eq!(result, tokens);
        }
    }

    mod tokeniser {
        use super::*;

        #[test]
        fn encode_decode_roundtrip_after_training() {
            let text = "hello world";
            let mut t = BpeTokeniser::new();
            t.train(text, 20);
            let ids    = t.encode(text);
            let decoded = t.decode(&ids);
            // Decoded should recover all characters (spaces handled via split_whitespace)
            // so words match; we check individual words
            assert!(decoded.contains("hello"), "decoded: {decoded}");
            assert!(decoded.contains("world"), "decoded: {decoded}");
        }

        #[test]
        fn training_increases_vocab_size_with_merges() {
            let mut t = BpeTokeniser::new();
            t.train("abcabc", 8); // base chars: a,b,c = 3; merges get us to 8
            assert!(t.vocab_size() > 3, "training should add merged tokens");
        }

        #[test]
        fn trained_tokeniser_produces_fewer_tokens_for_repeated_text() {
            // "aaaa" — after training "aa" is merged, so we need fewer tokens
            let text = "aaaa";
            let mut t = BpeTokeniser::new();
            let untrained_count = text.chars().count(); // 4 tokens without merges
            t.train(text, 5); // should learn "aa" merge
            let ids = t.encode(text);
            assert!(ids.len() < untrained_count,
                "trained tokeniser should use fewer tokens: {} vs {untrained_count}", ids.len());
        }

        #[test]
        fn encode_preserves_character_order() {
            let mut t = BpeTokeniser::new();
            t.train("hello", 10);
            let ids    = t.encode("hello");
            let decoded = t.decode(&ids);
            assert_eq!(decoded, "hello", "encode→decode should round-trip");
        }

        #[test]
        fn most_frequent_pair_is_merged_first() {
            // "ababab" → most frequent pair is ("a","b") with freq 3
            let mut t = BpeTokeniser::new();
            t.train("ababab", 5);
            assert!(!t.merges.is_empty(), "should have learned at least one merge");
            let (a, b) = &t.merges[0];
            assert_eq!(format!("{a}{b}"), "ab",
                "first merge should be 'ab' (most frequent pair)");
        }
    }
}
