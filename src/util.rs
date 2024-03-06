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

pub fn group_strings_to_fit_width(
    words: &[&str],
    max_width: usize,
    delimiter: &str,
) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut current_length: usize = 0;
    let mut current_group: Vec<String> = Vec::new();
    let delimiter_len = delimiter.len();
    for word in words {
        if !current_group.is_empty() && current_length + word.len() > max_width {
            groups.push(current_group);
            current_group = Vec::new();
            current_length = 0;
        }
        current_length += word.len();
        current_length += delimiter_len;
        current_group.push(word.to_string());
    }
    groups.push(current_group);
    groups
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

    #[test]
    fn test_group_strings_to_fit_width() {
        fn assert(actual: Vec<Vec<String>>, expected: &[&[&str]]) {
            assert_eq!(actual, expected);
        }

        let words = vec![];
        let actual = group_strings_to_fit_width(&words, 10, "");
        assert(actual, &[&[]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 2, "");
        assert(actual, &[&["aaa"], &["bbb"], &["ccc"], &["ddd"], &["eee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 4, "");
        assert(actual, &[&["aaa"], &["bbb"], &["ccc"], &["ddd"], &["eee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 6, "");
        assert(actual, &[&["aaa", "bbb"], &["ccc", "ddd"], &["eee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 8, "");
        assert(actual, &[&["aaa", "bbb"], &["ccc", "ddd"], &["eee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 9, "");
        assert(actual, &[&["aaa", "bbb", "ccc"], &["ddd", "eee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 15, "");
        assert(actual, &[&["aaa", "bbb", "ccc", "ddd", "eee"]]);

        let words = vec!["a", "b", "cc", "d", "ee"];
        let actual = group_strings_to_fit_width(&words, 3, "");
        assert(actual, &[&["a", "b"], &["cc", "d"], &["ee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 7, "--");
        assert(actual, &[&["aaa"], &["bbb"], &["ccc"], &["ddd"], &["eee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 8, "--");
        assert(actual, &[&["aaa", "bbb"], &["ccc", "ddd"], &["eee"]]);

        let words = vec!["aaa", "bbb", "ccc", "ddd", "eee"];
        let actual = group_strings_to_fit_width(&words, 9, "--");
        assert(actual, &[&["aaa", "bbb"], &["ccc", "ddd"], &["eee"]]);

        let words = vec!["a", "b", "c", "d", "e"];
        let actual = group_strings_to_fit_width(&words, 7, "     ");
        assert(actual, &[&["a", "b"], &["c", "d"], &["e"]]);
    }
}
