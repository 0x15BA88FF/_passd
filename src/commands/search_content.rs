pub fn filter_lines<F>(input: &str, filter_fn: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    input
        .lines()
        .filter(|line| filter_fn(line))
        .map(|line| line.to_string())
        .collect()
}
