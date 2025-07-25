pub fn parse_github_url(url: &str) -> Vec<&str> {
    let trimmed = &url["https://github.com/".len()..];
    let trimmed = trimmed.trim_end_matches(".git");
    let parts: Vec<&str> = trimmed.split('/').collect();
    return parts;
}

pub fn json_as_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}