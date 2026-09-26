/// Escape character used for every `ILIKE`/`LIKE` pattern built by
/// [`build_contains_pattern`].
///
/// PostgreSQL already defaults to a backslash, but passing it explicitly keeps
/// the pattern and the generated `ESCAPE` clause in sync no matter which
/// backend the query is compiled against.
pub const LIKE_ESCAPE_CHAR: char = '\\';

/**
 * Build a case-insensitive "contains" pattern out of a user supplied keyword.
 *
 * The keyword is **not** a SQL fragment, it always ends up as a bind parameter,
 * so this is not about SQL injection. What it does protect is the `LIKE`
 * metacharacters: a raw `%` would match every row, a raw `_` would match any
 * character, so searching for `100%` or `a_b` would silently return unrelated
 * results. Every metacharacter is prefixed with [`LIKE_ESCAPE_CHAR`] so the
 * database treats it as a literal.
 *
 * Must be paired with the escape clause on the query itself:
 *
 * ```ignore
 * use crate::common::utils::like_escape::{build_contains_pattern, LIKE_ESCAPE_CHAR};
 * use diesel::expression_methods::EscapeExpressionMethods;
 *
 * if let Some(pattern) = build_contains_pattern(keyword) {
 *     query = query.filter(table::name.ilike(pattern).escape(LIKE_ESCAPE_CHAR));
 * }
 * ```
 *
 * Returns `None` when the keyword is absent or blank, so callers can skip the
 * filter entirely and keep the unfiltered (and much cheaper) query.
 */
pub fn build_contains_pattern(keyword: Option<&String>) -> Option<String> {
    let word = keyword.map(|item| item.trim()).unwrap_or_default();
    if word.is_empty() {
        return None;
    }
    let mut pattern = String::with_capacity(word.len() + 2);
    pattern.push('%');
    for ch in word.chars() {
        if ch == '%' || ch == '_' || ch == LIKE_ESCAPE_CHAR {
            pattern.push(LIKE_ESCAPE_CHAR);
        }
        pattern.push(ch);
    }
    pattern.push('%');
    return Some(pattern);
}

#[cfg(test)]
mod tests {
    use super::{build_contains_pattern, LIKE_ESCAPE_CHAR};

    fn pattern(keyword: &str) -> Option<String> {
        build_contains_pattern(Some(&keyword.to_string()))
    }

    #[test]
    fn wraps_plain_keyword_with_wildcards() {
        assert_eq!(pattern("thesis").as_deref(), Some("%thesis%"));
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(pattern("  spaced  ").as_deref(), Some("%spaced%"));
    }

    #[test]
    fn returns_none_for_blank_or_missing_keyword() {
        assert_eq!(pattern(""), None);
        assert_eq!(pattern("   \t\n "), None);
        assert_eq!(build_contains_pattern(None), None);
    }

    #[test]
    fn escapes_like_metacharacters() {
        assert_eq!(pattern("100%").as_deref(), Some("%100\\%%"));
        assert_eq!(pattern("a_b").as_deref(), Some("%a\\_b%"));
        assert_eq!(pattern("%").as_deref(), Some("%\\%%"));
        assert_eq!(pattern("_").as_deref(), Some("%\\_%"));
    }

    #[test]
    fn escapes_the_escape_character_itself() {
        assert_eq!(pattern("back\\slash").as_deref(), Some("%back\\\\slash%"));
    }

    #[test]
    fn escapes_every_metacharacter_in_one_pass() {
        assert_eq!(pattern("50%_off\\").as_deref(), Some("%50\\%\\_off\\\\%"));
    }

    #[test]
    fn only_the_surrounding_wildcards_stay_unescaped() {
        let built = pattern("100%").unwrap();
        let inner = &built[1..built.len() - 1];
        assert_eq!(inner.matches(LIKE_ESCAPE_CHAR).count(), 1);
    }
}
