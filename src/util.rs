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
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(vec![], 10, "", &[])]
    #[case(vec![("a", 0), ("b", 0)], 0, "", &[])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 10, "", &["aa", "bbb", "cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 9, "", &["aa", "bbb", "cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 8, "", &["bbb", "cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 5, "", &["cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 3, "", &[])]
    #[case(vec![("ddd", 0), ("bbb", 0), ("ccc", 0), ("aaa", 0), ("eee", 0)], 10, "", &["ccc", "aaa", "eee"])]
    #[case(vec![("ddd", 0), ("bbb", 1), ("ccc", 1), ("aaa", 1), ("eee", 0)], 10, "", &["ddd", "aaa", "eee"])]
    #[case(vec![("ddd", 4), ("bbb", 3), ("ccc", 2), ("aaa", 1), ("eee", 0)], 10, "", &["ccc", "aaa", "eee"])]
    #[case(vec![("ddd", 0), ("bbb", 1), ("ccc", 2), ("aaa", 3), ("eee", 4)], 10, "", &["ddd", "bbb", "ccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 13, "--", &["aa", "bbb", "cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 12, "--", &["bbb", "cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 9, "--", &["bbb", "cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 8, "--", &["cccc"])]
    #[case(vec![("aa", 0), ("bbb", 0), ("cccc", 0)], 6, "--", &["cccc"])]
    #[case(vec![("a", 0), ("b", 0), ("c", 0)], 7, "     ", &["b", "c"])]
    #[trace]
    fn test_prune_strings_to_fit_width(
        #[case] words_with_priority: Vec<(&str, usize)>,
        #[case] max_width: usize,
        #[case] delimiter: &str,
        #[case] expected: &[&str],
    ) {
        let actual = prune_strings_to_fit_width(&words_with_priority, max_width, delimiter);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(vec![], 10, "", vec![vec![]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 2, "", vec![vec!["aaa"], vec!["bbb"], vec!["ccc"], vec!["ddd"], vec!["eee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 4, "", vec![vec!["aaa"], vec!["bbb"], vec!["ccc"], vec!["ddd"], vec!["eee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 6, "", vec![vec!["aaa", "bbb"], vec!["ccc", "ddd"], vec!["eee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 8, "", vec![vec!["aaa", "bbb"], vec!["ccc", "ddd"], vec!["eee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 9, "", vec![vec!["aaa", "bbb", "ccc"], vec!["ddd", "eee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 15, "", vec![vec!["aaa", "bbb", "ccc", "ddd", "eee"]])]
    #[case(vec!["a", "b", "cc", "d", "ee"], 3, "", vec![vec!["a", "b"], vec!["cc", "d"], vec!["ee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 7, "--", vec![vec!["aaa"], vec!["bbb"], vec!["ccc"], vec!["ddd"], vec!["eee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 8, "--", vec![vec!["aaa", "bbb"], vec!["ccc", "ddd"], vec!["eee"]])]
    #[case(vec!["aaa", "bbb", "ccc", "ddd", "eee"], 9, "--", vec![vec!["aaa", "bbb"], vec!["ccc", "ddd"], vec!["eee"]])]
    #[case(vec!["a", "b", "c", "d", "e"], 7, "     ", vec![vec!["a", "b"], vec!["c", "d"], vec!["e"]])]
    #[trace]
    fn test_group_strings_to_fit_width(
        #[case] words: Vec<&str>,
        #[case] max_width: usize,
        #[case] delimiter: &str,
        #[case] expected: Vec<Vec<&str>>,
    ) {
        let actual = group_strings_to_fit_width(&words, max_width, delimiter);
        assert_eq!(actual, expected);
    }
}
