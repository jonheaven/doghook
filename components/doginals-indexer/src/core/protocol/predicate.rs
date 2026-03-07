use config::DoginalsPredicatesConfig;
use doginals::inscription::Inscription;

/// Hiro-style predicate-driven selective indexing for Doginals (matches Chainhook/Ordhook design).
///
/// When `enabled = false` (default), every inscription passes. When enabled, an inscription
/// must satisfy ALL non-empty filter lists to be indexed.
pub fn inscription_matches_predicates(
    inscription: &Inscription,
    predicates: &DoginalsPredicatesConfig,
) -> bool {
    if !predicates.enabled {
        return true;
    }

    // mime_type filter: inscription content-type must start with one of the allowed types
    if !predicates.mime_types.is_empty() {
        let content_type = inscription
            .content_type
            .as_deref()
            .and_then(|b| std::str::from_utf8(b).ok())
            .unwrap_or("");
        if !predicates
            .mime_types
            .iter()
            .any(|m| content_type.starts_with(m.as_str()))
        {
            return false;
        }
    }

    // content_prefix filter: inscription body must start with one of the allowed UTF-8 prefixes
    if !predicates.content_prefixes.is_empty() {
        let body_str = inscription
            .body
            .as_deref()
            .and_then(|b| std::str::from_utf8(b).ok())
            .unwrap_or("");
        if !predicates
            .content_prefixes
            .iter()
            .any(|p| body_str.starts_with(p.as_str()))
        {
            return false;
        }
    }

    true
}
