//! Tiny fzf-style fuzzy matcher for the profile switcher.
//!
//! Profiles are a handful of short names, so a greedy subsequence match with
//! a few bonuses ranks them well enough; a crate would be more machinery than
//! the list it sorts.

/// Matched at the very first character: "umb" → "Umbra".
const PREFIX_BONUS: i32 = 8;
/// Matched at the start of a word: "dm" → "Dark Mode".
const WORD_START_BONUS: i32 = 6;
/// Matched right after the previous match: rewards contiguous runs.
const CONSECUTIVE_BONUS: i32 = 5;
const MATCH_SCORE: i32 = 1;

/// Case-insensitive subsequence score, or `None` when `query` is not a
/// subsequence of `candidate`. Higher is better. An empty query matches
/// everything with score 0.
pub fn score(query: &str, candidate: &str) -> Option<i32> {
    let candidate: Vec<char> = candidate.chars().collect();
    let mut total = 0;
    let mut position = 0;
    let mut previous_match: Option<usize> = None;

    for wanted in query.chars().filter(|c| !c.is_whitespace()) {
        let found = (position..candidate.len()).find(|&i| chars_equal(candidate[i], wanted))?;

        total += MATCH_SCORE;
        if found == 0 {
            total += PREFIX_BONUS;
        } else if is_word_start(&candidate, found) {
            total += WORD_START_BONUS;
        }
        match previous_match {
            Some(previous) if previous + 1 == found => total += CONSECUTIVE_BONUS,
            // A gap costs a little per skipped character, capped so one long
            // name is not buried below a much worse match.
            Some(previous) => total -= ((found - previous - 1) as i32).min(3),
            None => {}
        }

        previous_match = Some(found);
        position = found + 1;
    }

    Some(total)
}

/// Indices of `candidates` that match `query`, best first. Ties go to the
/// shorter name, then to the original order.
pub fn rank<S: AsRef<str>>(query: &str, candidates: &[S]) -> Vec<usize> {
    let mut scored: Vec<(usize, i32, usize)> = candidates
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let c = c.as_ref();
            score(query, c).map(|s| (i, s, c.chars().count()))
        })
        .collect();

    if query.trim().is_empty() {
        return scored.into_iter().map(|(i, _, _)| i).collect();
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)).then(a.0.cmp(&b.0)));
    scored.into_iter().map(|(i, _, _)| i).collect()
}

fn chars_equal(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

fn is_word_start(chars: &[char], index: usize) -> bool {
    let previous = chars[index - 1];
    let current = chars[index];
    !previous.is_alphanumeric() || (previous.is_lowercase() && current.is_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsequence_matches_case_insensitively() {
        assert!(score("umb", "Umbra").is_some());
        assert!(score("UMB", "umbra").is_some());
        assert!(score("dm", "Dark Mode").is_some());
        assert!(score("xyz", "Umbra").is_none());
        // Order matters: a subsequence, not a bag of letters.
        assert!(score("bmu", "Umbra").is_none());
    }

    #[test]
    fn empty_query_keeps_the_original_order() {
        let names = ["Light Mode", "Dark Mode", "Umbra"];
        assert_eq!(rank("", &names), vec![0, 1, 2]);
        assert_eq!(rank("  ", &names), vec![0, 1, 2]);
    }

    #[test]
    fn prefix_beats_a_match_buried_in_another_word() {
        let names = ["Dark Mode (umber)", "Umbra"];
        assert_eq!(rank("umb", &names), vec![1, 0]);
    }

    #[test]
    fn word_starts_beat_scattered_letters() {
        // "dm" hits both word starts of "Dark Mode" but is scattered in
        // "Random Mist".
        let names = ["Random Mist", "Dark Mode"];
        assert_eq!(rank("dm", &names)[0], 1);
    }

    #[test]
    fn non_matches_are_dropped_and_ties_prefer_shorter_names() {
        let names = ["Light Mode", "Light", "Dark"];
        assert_eq!(rank("light", &names), vec![1, 0]);
    }

    #[test]
    fn camel_case_humps_count_as_word_starts() {
        assert!(score("pwa", "PartialWithActive").unwrap() > score("pwa", "Pawpaw").unwrap());
        assert_eq!(rank("pwa", &["PartialWithActive", "pawn"]), vec![0]);
    }
}
