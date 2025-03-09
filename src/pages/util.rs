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
