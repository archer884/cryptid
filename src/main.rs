// Reference: https://github.com/davidkellis/cryptogram/blob/master/src/cryptogram.cr
// David's cryptogram solver.

extern crate fxhash;
extern crate stopwatch;

use fxhash::{FxHashMap, FxHashSet};
use std::collections::VecDeque;
use std::borrow::Cow;

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

#[derive(Debug, Default)]
struct Solver<'words> {
    words_by_length: FxHashMap<usize, FxHashSet<&'words str>>,
    words_by_character_and_index: FxHashMap<usize, FxHashMap<u8, FxHashSet<&'words str>>>,
}

impl<'words> Solver<'words> {
    fn from_dictionary(words: &[&'words str]) -> Self {
        let mut solver = Solver::default();

        for &word in words {
            solver
                .words_by_length
                .entry(word.len())
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

    // Never used a clone-on-write pointer like this before...
    //
    // Honestly, all things considered, this works amazingly well. Rust is even able to guess the 
    // appropriate default type!
    fn words_by_character_and_index(&self, u: u8, idx: usize) -> Cow<FxHashSet<&'words str>> {
        self.words_by_character_and_index
            .get(&idx)
            .and_then(|by_char| by_char.get(&u))
            .map(Cow::Borrowed)
            .unwrap_or_default()
    }

    fn words_by_length(&self, len: usize) -> Cow<FxHashSet<&'words str>> {
        self.words_by_length
            .get(&len)
            .map(Cow::Borrowed)
            .unwrap_or_default()
    }

    fn solve(&self, phrase: &Phrase) -> Vec<String> {
        let phrase = phrase.as_ref();

        // FIXME: this part is only going to work for "properly" formatted cryptograms--which is
        // to say the kind that don't have punctuation or other non-letter characters.
        //
        // It probably also looks weird to use a VecDeque instead of just a Vec, but the fact is
        // I don't get why David is popping from the front ("shift?" in his code) and I don't
        // want to change it.
        let encrypted_words: VecDeque<_> = phrase.split_whitespace().collect();

        // It looks strange to consume this vector here, but that's intentional. David is
        // definitely eating this vector. He's also cloning bits and pieces of it across stack
        // frames, so I guess just be happy these are string slices instead of strings.
        let letter_mappings = self.guess(FxHashMap::default(), encrypted_words);

        unimplemented!()
    }

    // According to David, this is where the magic happens. I think that means this is the method
    // I'm never gonna be able to port. Unanswered questions include: does the hash map here need 
    // be passed by unique reference or by value? I guess the same question applies above.
    fn guess(
        &self,
        mut mapping: FxHashMap<u8, u8>,
        encrypted_words: VecDeque<&str>,
    ) -> Vec<FxHashMap<u8, u8>> {
        let mut encrypted_words = encrypted_words.clone();
        match encrypted_words.pop_front() {
            None => vec![mapping],
            Some(encrypted_word) => {
                let words = self.find_candidate_matches(encrypted_word, &mut mapping);
                
                unimplemented!("Pick up at line 51");
            }
        }
    }

    // FIXME: I can only guess what this method does because the syntax for this isn't exactly
    // self-evident in the original code. In particular, at this point, I'm not sure if this
    // method should be eating the hash map or referencing it.
    fn find_candidate_matches(&self, word: &str, mapping: &FxHashMap<u8, u8>) -> FxHashSet<&'words str> {
        let mut candidates = self.words_by_length(word.len()).into_owned();
        
        for (idx, u) in word.bytes().enumerate() {
            if let Some(&mapped_char) = mapping.get(&u) {
                let other_candidates = self.words_by_character_and_index(mapped_char, idx);
                candidates.retain(|x| other_candidates.contains(x));
            }
        }

        // It strikes me that David's code might be paring down the original candidate set rather
        // than a copy of it, which is what I have here. I don't know enough about his 
        // implementation to guess whether or not that is intended or correct or whatever.
        candidates
    }
}

fn main() {
    use std::env;

    // Enable1.txt does not include words like A or I. It may be prefereable to employ a custom
    // word list or, alternatively, /usr/share/dict/words
    let words: Vec<_> = include_str!("../resources/enable1.txt")
        .split_whitespace()
        .collect();

    let (elapsed, solver) = time!(Solver::from_dictionary(&words));
    println!("Mapping time: {:?}", elapsed); // ~300 milliseconds

    for phrase in env::args().skip(1).filter_map(Phrase::from_str) {
        let (elapsed, solutions) = time!(solver.solve(&phrase));
        for solution in solutions {
            println!("{}", solution);
        }
        println!("Elapsed: {:?}", elapsed);
    }

    println!("Hello, world!");
}
