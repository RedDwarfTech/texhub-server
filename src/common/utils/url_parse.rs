pub fn parse_github_url(url: &str) -> Vec<&str> {
    let trimmed = &url["https://github.com/".len()..];
    let trimmed = trimmed.trim_end_matches(".git");
    let parts: Vec<&str> = trimmed.split('/').collect();
    return parts;
}
