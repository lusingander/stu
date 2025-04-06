pub fn prune_strings_to_fit_width(
    words_with_priority: &[(String, usize)],
    max_width: usize,
    delimiter: &str,
) -> Vec<String> {
    let words_total_length = words_with_priority
        .iter()
        .map(|(s, _)| console::measure_text_width(s))
        .sum::<usize>();
    let delimiter_len = console::measure_text_width(delimiter);
    let delimiter_total_length = words_with_priority.len().saturating_sub(1) * delimiter_len;
    let mut total_length = words_total_length + delimiter_total_length;

    let mut words_with_priority_with_index: Vec<(usize, &(String, usize))> =
        words_with_priority.iter().enumerate().collect();

    words_with_priority_with_index.sort_by(|(_, (_, p1)), (_, (_, p2))| p2.cmp(p1));

    let mut prune: Vec<usize> = Vec::new();
    for (i, (s, _)) in &words_with_priority_with_index {
        if total_length <= max_width {
            break;
        }
        prune.push(*i);
        total_length -= console::measure_text_width(s);
        total_length -= delimiter_len;
    }

    words_with_priority
        .iter()
        .enumerate()
        .filter(|(i, _)| !prune.contains(i))
        .map(|(_, (s, _))| s.to_string())
        .collect()
}

pub fn digits(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut n = n;
    let mut c = 0;
    while n > 0 {
        n /= 10;
        c += 1;
    }
    c
}

pub fn extension_from_file_name(filename: &str) -> String {
    filename
        .split('.')
        .next_back()
        .map(|s| s.to_string())
        .unwrap_or_default()
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
        let words_with_priority: Vec<(String, usize)> = words_with_priority
            .into_iter()
            .map(|(s, n)| (s.to_owned(), n))
            .collect();
        let actual = prune_strings_to_fit_width(&words_with_priority, max_width, delimiter);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_digits() {
        assert_eq!(digits(0), 1);
        assert_eq!(digits(1), 1);
        assert_eq!(digits(30), 2);
        assert_eq!(digits(123), 3);
        assert_eq!(digits(9999), 4);
        assert_eq!(digits(10000), 5);
    }

    #[test]
    fn test_extension_from_file_name() {
        assert_eq!(extension_from_file_name("a.txt"), "txt");
        assert_eq!(extension_from_file_name("a.gif.txt"), "txt");
    }
}
