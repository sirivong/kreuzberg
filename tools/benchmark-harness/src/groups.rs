//! Fast benchmark groups: curated document subsets for targeted iteration.

use std::path::Path;

use crate::Result;
use crate::corpus::{CorpusDocument, CorpusFilter, build_corpus};

/// A named benchmark group backed by exact document IDs and/or fixture metadata.
pub struct BenchmarkGroup {
    pub name: &'static str,
    pub description: &'static str,
    /// Exact fixture stems. These are intentionally not substring patterns.
    pub docs: &'static [&'static str],
    /// Match any fixture whose `metadata.size_tier` is one of these values.
    pub size_tiers: &'static [&'static str],
    /// Match any fixture whose `metadata.role` is one of these values.
    pub roles: &'static [&'static str],
    /// Match any fixture containing one of these `metadata.cohorts` values.
    pub cohorts: &'static [&'static str],
}

pub const GROUPS: &[BenchmarkGroup] = &[
    BenchmarkGroup {
        name: "hotspot",
        description: "Maintained fast loop for current PDF quality hotspots",
        docs: &[
            "160428551",
            "2309.17020",
            "24231810",
            "681693",
            concat!("ft_A", "CN_2009_page_102_t0"),
            "pb_FBLB-134215544_page147",
            "pb_fqr-retail-blackrock-global-allocation-fund-inc_page4",
            "pb_sample_page_16_page1",
        ],
        size_tiers: &[],
        roles: &[],
        cohorts: &[],
    },
    BenchmarkGroup {
        name: "smoke",
        description: "Corpus-maintained smoke tier (metadata.size_tier=smoke)",
        docs: &[],
        size_tiers: &["smoke"],
        roles: &[],
        cohorts: &[],
    },
    BenchmarkGroup {
        name: "promotion",
        description: "Smoke tier plus held-out evaluation fixtures",
        docs: &[],
        size_tiers: &["smoke"],
        roles: &["eval"],
        cohorts: &[],
    },
    BenchmarkGroup {
        name: "tables",
        description: "All fixtures tagged with the tables cohort",
        docs: &[],
        size_tiers: &[],
        roles: &[],
        cohorts: &["tables"],
    },
    BenchmarkGroup {
        name: "structure",
        description: "All fixtures tagged with nested heading structure",
        docs: &[],
        size_tiers: &[],
        roles: &[],
        cohorts: &["nested-heading"],
    },
    BenchmarkGroup {
        name: "lists",
        description: "All fixtures tagged with nested lists",
        docs: &[],
        size_tiers: &[],
        roles: &[],
        cohorts: &["nested-list"],
    },
];

impl BenchmarkGroup {
    pub fn matches(&self, doc: &CorpusDocument) -> bool {
        self.docs.contains(&doc.name.as_str())
            || metadata_string_matches(&doc.metadata, "size_tier", self.size_tiers)
            || metadata_string_matches(&doc.metadata, "role", self.roles)
            || metadata_array_matches(&doc.metadata, "cohorts", self.cohorts)
    }
}

fn metadata_string_matches(
    metadata: &std::collections::HashMap<String, serde_json::Value>,
    key: &str,
    expected: &[&str],
) -> bool {
    !expected.is_empty()
        && metadata
            .get(key)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| expected.contains(&value))
}

fn metadata_array_matches(
    metadata: &std::collections::HashMap<String, serde_json::Value>,
    key: &str,
    expected: &[&str],
) -> bool {
    !expected.is_empty()
        && metadata
            .get(key)
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| {
                values
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .any(|value| expected.contains(&value))
            })
}

/// Resolve a group to exact fixture stems using current corpus metadata.
pub fn resolve_group_docs(fixtures_dir: &Path, group: &BenchmarkGroup) -> Result<Vec<String>> {
    let docs = build_corpus(
        fixtures_dir,
        &CorpusFilter {
            require_ground_truth: true,
            ..Default::default()
        },
    )?;
    let matches: Vec<String> = docs
        .into_iter()
        .filter(|doc| group.matches(doc))
        .map(|doc| doc.name)
        .collect();
    if matches.is_empty() {
        return Err(crate::Error::Config(format!(
            "benchmark group '{}' matched zero documents in {}",
            group.name,
            fixtures_dir.display()
        )));
    }
    Ok(matches)
}

/// Find a group by name, case-insensitive.
pub fn find_group(name: &str) -> Option<&'static BenchmarkGroup> {
    GROUPS.iter().find(|g| g.name.eq_ignore_ascii_case(name))
}

/// List all available group names.
pub fn group_names() -> Vec<&'static str> {
    GROUPS.iter().map(|g| g.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn doc(name: &str, metadata: serde_json::Value) -> CorpusDocument {
        CorpusDocument {
            name: name.to_string(),
            document_path: PathBuf::new(),
            file_type: "pdf".to_string(),
            file_size: 0,
            ground_truth_text: None,
            ground_truth_markdown: None,
            metadata: serde_json::from_value::<HashMap<String, serde_json::Value>>(metadata).unwrap(),
            fixture_path: PathBuf::new(),
        }
    }

    #[test]
    fn smoke_and_promotion_follow_metadata() {
        let smoke = find_group("smoke").unwrap();
        let promotion = find_group("promotion").unwrap();
        let tune_smoke = doc("a", serde_json::json!({"size_tier": "smoke", "role": "tune"}));
        let eval_core = doc("b", serde_json::json!({"size_tier": "core", "role": "eval"}));
        assert!(smoke.matches(&tune_smoke));
        assert!(!smoke.matches(&eval_core));
        assert!(promotion.matches(&tune_smoke));
        assert!(promotion.matches(&eval_core));
    }

    #[test]
    fn thematic_group_matches_cohort() {
        let tables = find_group("tables").unwrap();
        let table_doc = doc("a", serde_json::json!({"cohorts": ["native-clean", "tables"]}));
        assert!(tables.matches(&table_doc));
    }

    #[test]
    fn zero_match_group_is_an_error() {
        let fixtures = tempfile::tempdir().unwrap();
        let error = resolve_group_docs(fixtures.path(), find_group("smoke").unwrap()).unwrap_err();
        assert!(matches!(error, crate::Error::Config(_)));
    }
}
