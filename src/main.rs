// Reference: https://github.com/davidkellis/cryptogram/blob/master/src/cryptogram.cr
// David's cryptogram solver.

use hashbrown::{HashMap, HashSet};

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
        let mut symbol_map = HashMap::new();

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
    words_by_pattern: HashMap<Pattern, HashSet<&'words str>>,
    words_by_character_and_index: HashMap<usize, HashMap<u8, HashSet<&'words str>>>,
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

    fn words_by_pattern(&self, word: &str) -> HashSet<&'words str> {
        let pattern = Pattern::from_str(word);
        self.words_by_pattern
            .get(&pattern)
            .cloned()
            .unwrap_or_default()
    }

    fn words_by_character_and_index(&self, u: u8, idx: usize) -> Option<&HashSet<&'words str>> {
        self.words_by_character_and_index
            .get(&idx)
            .and_then(|by_char| by_char.get(&u))
    }

    // FIXME: use internal iteration to print solutions as they are discovered.
    fn solve<'a>(&self, phrase: &'a Phrase) -> impl Iterator<Item = String> + 'a {
        // FIXME: this part is only going to work for "properly" formatted cryptograms--which is
        // to say the kind that don't have punctuation or other non-letter characters.
        let encrypted_words: HashSet<_> = phrase.as_ref().split_whitespace().collect();
        let encrypted_words: Vec<_> = encrypted_words.into_iter().collect();
        let letter_mappings = self.guess(HashMap::new(), &encrypted_words);

        letter_mappings.into_iter().map(move |mapping| {
            phrase
                .as_ref()
                .bytes()
                .map(|u| mapping.get(&u).copied().unwrap_or(u) as char)
                .collect()
        })
    }

    fn guess(&self, mapping: HashMap<u8, u8>, encrypted_words: &[&str]) -> Vec<HashMap<u8, u8>> {
        use std::cmp::Reverse;

        let mut encrypted_words: Vec<_> = encrypted_words
            .iter()
            .map(|word| {
                let candidate_matches = self.find_candidate_matches(word, &mapping);
                (word, candidate_matches)
            })
            .collect();

        encrypted_words.sort_by_key(|pair| Reverse(pair.1.len()));

        match encrypted_words.pop() {
            None => vec![mapping],
            Some((encrypted_word, candidate_words)) => {
                let mut candidate_mappings = HashMap::new();

                for &word in &candidate_words {
                    if let Some(mapping) = self.try_extend_mapping(word, encrypted_word, &mapping) {
                        candidate_mappings.insert(word, mapping);
                    }
                }

                let encrypted_words: Vec<_> =
                    encrypted_words.iter().map(|&(&word, _)| word).collect();

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
        mapping: &HashMap<u8, u8>,
    ) -> HashSet<&'words str> {
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
        mapping: &HashMap<u8, u8>,
    ) -> Option<HashMap<u8, u8>> {
        let mut new_mapping = HashMap::new();

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
        let value_set: HashSet<u8> = new_mapping.values().cloned().collect();
        if value_set.len() != new_mapping.len() {
            return None;
        }

        Some(new_mapping)
    }
}

fn main() {
    use std::env;

    let phrase = env::args()
        .nth(1)
        .and_then(Phrase::from_str)
        .expect("Provide a phrase, would you?");

    // Enable1.txt does not include words like A or I. It may be preferable to employ a custom
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
