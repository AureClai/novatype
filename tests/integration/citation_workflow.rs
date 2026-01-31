//! Integration tests for citation workflows.

use nova_cite::{CitationManager, CitationStyle, Formatter};
use nova_cite::bibtex::Bibliography;

const SAMPLE_BIBTEX: &str = r#"
@article{einstein1905,
    author = {Albert Einstein},
    title = {On the Electrodynamics of Moving Bodies},
    journal = {Annalen der Physik},
    year = {1905},
    volume = {17},
    pages = {891--921}
}

@book{knuth1997,
    author = {Donald E. Knuth},
    title = {The Art of Computer Programming},
    publisher = {Addison-Wesley},
    year = {1997},
    volume = {1}
}

@inproceedings{turing1950,
    author = {Alan M. Turing},
    title = {Computing Machinery and Intelligence},
    booktitle = {Mind},
    year = {1950},
    volume = {59},
    pages = {433--460}
}
"#;

/// Test parsing a bibliography file.
#[test]
fn parse_bibliography() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();

    assert_eq!(bib.len(), 3);
    assert!(bib.contains("einstein1905"));
    assert!(bib.contains("knuth1997"));
    assert!(bib.contains("turing1950"));
}

/// Test accessing entry fields.
#[test]
fn access_entry_fields() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let entry = bib.get("einstein1905").unwrap();

    assert_eq!(entry.entry_type, "article");
    assert_eq!(entry.title(), Some("On the Electrodynamics of Moving Bodies"));
    assert_eq!(entry.author(), Some("Albert Einstein"));
    assert_eq!(entry.year(), Some("1905"));
}

/// Test citation formatting with IEEE style.
#[test]
fn format_ieee_citation() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let entry = bib.get("einstein1905").unwrap();

    let formatter = Formatter::new().with_style(CitationStyle::Ieee);
    let formatted = formatter.format_entry(entry).unwrap();

    assert!(formatted.contains("Einstein"));
    assert!(formatted.contains("Electrodynamics"));
    assert!(formatted.contains("1905"));
}

/// Test citation formatting with APA style.
#[test]
fn format_apa_citation() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let entry = bib.get("knuth1997").unwrap();

    let formatter = Formatter::new().with_style(CitationStyle::Apa);
    let formatted = formatter.format_entry(entry).unwrap();

    assert!(formatted.contains("Knuth"));
    assert!(formatted.contains("(1997)"));
}

/// Test citation manager with local bibliography.
#[tokio::test]
async fn citation_manager_local_resolve() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let manager = CitationManager::new().with_bibliography(bib);

    let entry = manager.resolve("einstein1905").await.unwrap();
    assert_eq!(entry.key, "einstein1905");
    assert_eq!(entry.title(), Some("On the Electrodynamics of Moving Bodies"));
}

/// Test citation manager with missing key.
#[tokio::test]
async fn citation_manager_not_found() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let manager = CitationManager::new().with_bibliography(bib);

    let result = manager.resolve("nonexistent").await;
    assert!(result.is_err());
}

/// Test citation caching.
#[tokio::test]
async fn citation_manager_caching() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let manager = CitationManager::new().with_bibliography(bib);

    // First resolve
    let _ = manager.resolve("einstein1905").await.unwrap();
    assert_eq!(manager.cache_size().await, 1);

    // Second resolve (should use cache)
    let _ = manager.resolve("einstein1905").await.unwrap();
    assert_eq!(manager.cache_size().await, 1);

    // Different key
    let _ = manager.resolve("knuth1997").await.unwrap();
    assert_eq!(manager.cache_size().await, 2);

    // Clear cache
    manager.clear_cache().await;
    assert_eq!(manager.cache_size().await, 0);
}

/// Test batch resolution.
#[tokio::test]
async fn citation_manager_batch_resolve() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let manager = CitationManager::new().with_bibliography(bib);

    let keys = vec!["einstein1905", "knuth1997", "nonexistent"];
    let results = manager.resolve_batch(&keys).await;

    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok());
    assert!(results[1].is_ok());
    assert!(results[2].is_err());
}

/// Test formatting with manager's default style.
#[tokio::test]
async fn citation_manager_format() {
    let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
    let manager = CitationManager::new()
        .with_bibliography(bib)
        .with_style(CitationStyle::Ieee);

    let entry = manager.resolve("turing1950").await.unwrap();
    let formatted = manager.format(&entry).unwrap();

    assert!(formatted.contains("Turing"));
    assert!(formatted.contains("Computing Machinery"));
}

/// Test style parsing.
#[test]
fn citation_style_from_name() {
    assert_eq!(CitationStyle::from_name("ieee").unwrap(), CitationStyle::Ieee);
    assert_eq!(CitationStyle::from_name("APA").unwrap(), CitationStyle::Apa);
    assert_eq!(CitationStyle::from_name("chicago").unwrap(), CitationStyle::Chicago);

    assert!(CitationStyle::from_name("unknown_style").is_err());
}
