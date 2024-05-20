pub fn build_helps(helps: &[(&[&str], &str)]) -> Vec<String> {
    helps
        .iter()
        .map(|(keys, desc)| {
            let key_maps = keys
                .iter()
                .map(|key| format!("<{}>", key))
                .collect::<Vec<String>>()
                .join(" ");
            let help = format!("{}: {}", key_maps, desc);
            help
        })
        .collect()
}

pub fn build_short_helps(helps: &[(&[&str], &str, usize)]) -> Vec<(String, usize)> {
    helps
        .iter()
        .map(|(keys, desc, priority)| {
            let key_maps = keys
                .iter()
                .map(|key| format!("<{}>", key))
                .collect::<Vec<String>>()
                .join(" ");
            let help = format!("{}: {}", key_maps, desc);
            (help, *priority)
        })
        .collect()
}
