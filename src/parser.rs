pub fn parse_exit_code(source: &str) -> Option<i32> {
    let trimmed = source.trim();

    if trimmed.starts_with("exit(") && trimmed.ends_with(");") {
        let inner = &trimmed[5..trimmed.len() - 2];
        return inner.parse::<i32>().ok();
    }
    None
}
