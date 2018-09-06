// Reference: https://github.com/davidkellis/cryptogram/blob/master/src/cryptogram.cr
// David's cryptogram solver.

extern crate fxhash;
extern crate stopwatch;

use fxhash::{FxHashMap, FxHashSet};

macro_rules! time {
    ($e:expr) => {{
        let mut time = stopwatch::Stopwatch::start_new();
        let result = $e;
        time.stop();
        (time.elapsed(), result)
    }};
}

/// Represents a phrase to be solved.
///
/// A phrase differs from an ordinary string in that a phrase is guaranteed to be lowercase
/// ascii text.
#[derive(Debug)]
struct Phrase(String);

impl Phrase {
    fn from_str(s: impl AsRef<str>) -> Option<Phrase> {
        let s = s.as_ref();
        if s.is_ascii() {
            Some(Phrase(s.to_ascii_lowercase()))
        } else {
            None
        }
    }
}

impl AsRef<str> for Phrase {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct Pattern(Vec<u8>);

impl Pattern {
    fn from_str(s: &str) -> Self {
        let mut next_symbol = 0;
        let mut symbols = Vec::new();
        let mut symbol_map = FxHashMap::default();

        for u in s.bytes() {
            symbols.push(*symbol_map.entry(u).or_insert_with(|| {
                let insert = next_symbol;
                next_symbol += 1;
                insert
            }));
        }

        Pattern(symbols)
    }
}

#[derive(Debug, Default)]
struct Solver<'words> {
    words_by_pattern: FxHashMap<Pattern, FxHashSet<&'words str>>,
    words_by_character_and_index: FxHashMap<usize, FxHashMap<u8, FxHashSet<&'words str>>>,
}

impl<'words> Solver<'words> {
    fn from_dictionary(words: &[&'words str]) -> Self {
        let mut solver = Solver::default();

        for &word in words {
            solver
                .words_by_pattern
                .entry(Pattern::from_str(word))
                .or_default()
                .insert(word);

            for (idx, u) in word.bytes().enumerate() {
                solver
                    .words_by_character_and_index
                    .entry(idx)
                    .or_default()
                    .entry(u)
                    .or_default()
                    .insert(word);
            }
        }

        solver
    }

    fn words_by_pattern(&self, word: &str) -> FxHashSet<&'words str> {
        let pattern = Pattern::from_str(word);
        self.words_by_pattern
            .get(&pattern)
            .map(|x| x.clone())
            .unwrap_or_default()
    }

    fn words_by_character_and_index(&self, u: u8, idx: usize) -> Option<&FxHashSet<&'words str>> {
        self.words_by_character_and_index
            .get(&idx)
            .and_then(|by_char| by_char.get(&u))
    }

    fn solve<'a>(&self, phrase: &'a Phrase) -> impl Iterator<Item = String> + 'a {
        // FIXME: this part is only going to work for "properly" formatted cryptograms--which is
        // to say the kind that don't have punctuation or other non-letter characters.
        let encrypted_words = {
            let mut encrypted_words: FxHashSet<_> = phrase.as_ref().split_whitespace().collect();
            encrypted_words.into_iter().collect()
        };

        let letter_mappings = self.guess(FxHashMap::default(), &encrypted_words);

        letter_mappings.into_iter().map(move |mapping| {
            phrase
                .as_ref()
                .bytes()
                .map(|u| mapping.get(&u).map(|&u| u).unwrap_or(u) as char)
                .collect()
        })
    }

    // FIXME: in this method, we calculate candidate matches for the target word twice when we
    // could get away with doing it just once and reduce the amount of work done by some
    // minor degree. >.>
    fn guess(
        &self,
        mut mapping: FxHashMap<u8, u8>,
        encrypted_words: &Vec<&str>,
    ) -> Vec<FxHashMap<u8, u8>> {
        use std::cmp::Reverse;

        let mut encrypted_words: Vec<_> = encrypted_words.into_iter().cloned().collect();
        encrypted_words
            .sort_by_key(|word| Reverse(self.find_candidate_matches(word, &mapping).len()));

        match encrypted_words.pop() {
            None => vec![mapping],
            Some(encrypted_word) => {
                let candidate_words = self.find_candidate_matches(encrypted_word, &mut mapping);
                let mut candidate_mappings = FxHashMap::default();

                for &word in &candidate_words {
                    if let Some(mapping) = self.try_extend_mapping(word, encrypted_word, &mapping) {
                        candidate_mappings.insert(word, mapping);
                    }
                }

                candidate_mappings
                    .into_iter()
                    .flat_map(move |(_, mapping)| self.guess(mapping, &encrypted_words))
                    .collect()
            }
        }
    }

    fn find_candidate_matches(
        &self,
        word: &str,
        mapping: &FxHashMap<u8, u8>,
    ) -> FxHashSet<&'words str> {
        let mut candidates = self.words_by_pattern(word);

        for (idx, u) in word.bytes().enumerate() {
            if let Some(&mapped_char) = mapping.get(&u) {
                if let Some(other_candidates) = self.words_by_character_and_index(mapped_char, idx)
                {
                    candidates.retain(|x| other_candidates.contains(x));
                }
            }
        }

        candidates
    }

    /// Attempts to extend mapping based on an encrypted word and a candidate solution.
    fn try_extend_mapping(
        &self,
        word: &str,
        encrypted_word: &str,
        mapping: &FxHashMap<u8, u8>,
    ) -> Option<FxHashMap<u8, u8>> {
        let mut new_mapping = FxHashMap::default();

        for (u_encoded, u_decoded) in encrypted_word.bytes().zip(word.bytes()) {
            if let Some(&mapped_char) = new_mapping.get(&u_encoded) {
                if mapped_char != u_decoded {
                    return None;
                }
            }

            if let Some(&mapped_char) = mapping.get(&u_encoded) {
                if mapped_char != u_decoded {
                    return None;
                }
            }

            new_mapping.insert(u_encoded, u_decoded);
        }

        mapping.iter().for_each(|(&k, &v)| {
            // This weirdness should avoid me re-inserting anything already in the map, which will
            // in turn avoid overwriting anything by mistake. Although I think that should be
            // impossible because of the code above.
            new_mapping.entry(k).or_insert(v);
        });

        // Test for mistakenly mapping multiple characters to one character.
        let value_set: FxHashSet<u8> = new_mapping.values().cloned().collect();
        if value_set.len() != new_mapping.len() {
            return None;
        }

        Some(new_mapping)
    }
}

fn main() {
    use std::env;
    use std::process;

    let phrase = env::args()
        .nth(1)
        .and_then(Phrase::from_str)
        .expect("Provide a phrase, would you?");

    // Enable1.txt does not include words like A or I. It may be prefereable to employ a custom
    // word list or, alternatively, /usr/share/dict/words
    let words: Vec<_> = include_str!("../resources/enable1.txt")
        .split_whitespace()
        .collect();

    let (elapsed, solver) = time!(Solver::from_dictionary(&words));
    println!("Initialize: {:?}", elapsed);

    let (elapsed, mut solutions) = time!(solver.solve(&phrase).collect::<Vec<_>>());
    solutions.sort();
    solutions
        .iter()
        .for_each(|solution| println!("{}", solution));
        
    println!("Elapsed: {:?}", elapsed);
}
