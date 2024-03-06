pub fn to_preview_string(bytes: &[u8], _content_type: &str) -> String {
    // fixme: consider content_type
    String::from_utf8_lossy(bytes).into()
}

pub fn prune_strings_to_fit_width(
    words_with_priority: &[(&str, usize)],
    max_width: usize,
    delimiter: &str,
) -> Vec<String> {
    let words_total_length = words_with_priority
        .iter()
        .map(|(s, _)| s.len())
        .sum::<usize>();
    let delimiter_total_length = words_with_priority.len().saturating_sub(1) * delimiter.len();
    let mut total_length = words_total_length + delimiter_total_length;

    let mut words_with_priority_with_index: Vec<(usize, &(&str, usize))> =
        words_with_priority.iter().enumerate().collect();

    words_with_priority_with_index.sort_by(|(_, (_, p1)), (_, (_, p2))| p2.cmp(p1));

    let mut prune: Vec<usize> = Vec::new();
    for (i, (s, _)) in &words_with_priority_with_index {
        if total_length <= max_width {
            break;
        }
        prune.push(*i);
        total_length -= s.len();
        total_length -= delimiter.len();
    }

    words_with_priority
        .iter()
        .enumerate()
        .filter(|(i, _)| !prune.contains(i))
        .map(|(_, (s, _))| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_strings_to_fit_width() {
        fn assert(actual: Vec<String>, expected: &[&str]) {
            assert_eq!(actual, expected);
        }

        let words_with_priority = vec![];
        let actual = prune_strings_to_fit_width(&words_with_priority, 10, "");
        assert(actual, &[]);

        let words_with_priority = vec![("a", 0), ("b", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 0, "");
        assert(actual, &[]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 10, "");
        assert(actual, &["aa", "bbb", "cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 9, "");
        assert(actual, &["aa", "bbb", "cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 8, "");
        assert(actual, &["bbb", "cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 5, "");
        assert(actual, &["cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 3, "");
        assert(actual, &[]);

        let words_with_priority = vec![("ddd", 0), ("bbb", 0), ("ccc", 0), ("aaa", 0), ("eee", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 10, "");
        assert(actual, &["ccc", "aaa", "eee"]);

        let words_with_priority = vec![("ddd", 0), ("bbb", 1), ("ccc", 1), ("aaa", 1), ("eee", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 10, "");
        assert(actual, &["ddd", "aaa", "eee"]);

        let words_with_priority = vec![("ddd", 4), ("bbb", 3), ("ccc", 2), ("aaa", 1), ("eee", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 10, "");
        assert(actual, &["ccc", "aaa", "eee"]);

        let words_with_priority = vec![("ddd", 0), ("bbb", 1), ("ccc", 2), ("aaa", 3), ("eee", 4)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 10, "");
        assert(actual, &["ddd", "bbb", "ccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 13, "--");
        assert(actual, &["aa", "bbb", "cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 12, "--");
        assert(actual, &["bbb", "cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 9, "--");
        assert(actual, &["bbb", "cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 8, "--");
        assert(actual, &["cccc"]);

        let words_with_priority = vec![("aa", 0), ("bbb", 0), ("cccc", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 6, "--");
        assert(actual, &["cccc"]);

        let words_with_priority = vec![("a", 0), ("b", 0), ("c", 0)];
        let actual = prune_strings_to_fit_width(&words_with_priority, 7, "     ");
        assert(actual, &["b", "c"]);
    }
}
