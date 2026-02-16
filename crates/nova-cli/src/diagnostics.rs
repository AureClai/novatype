//! Rich diagnostic rendering for compilation errors.
//!
//! Converts [`NovaDiagnostic`] from nova-core into miette-renderable
//! diagnostics with source snippets, colors, and help text.

use miette::{GraphicalReportHandler, LabeledSpan, MietteDiagnostic, NamedSource, Severity};
use novatype_core::{DiagnosticSeverity, NovaDiagnostic};

/// Render a list of compilation errors to stderr and return an anyhow error.
pub fn render_errors(diags: &[NovaDiagnostic]) -> anyhow::Result<()> {
    let handler = GraphicalReportHandler::new();

    for diag in diags {
        render_one(&handler, diag);
    }

    anyhow::bail!(
        "compilation failed with {} error(s)",
        diags
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    )
}

/// Render a single warning diagnostic to stderr.
pub fn render_warning(diag: &NovaDiagnostic) {
    let handler = GraphicalReportHandler::new();
    render_one(&handler, diag);
}

/// Render a single diagnostic to stderr using miette.
fn render_one(handler: &GraphicalReportHandler, diag: &NovaDiagnostic) {
    let severity = match diag.severity {
        DiagnosticSeverity::Error => Severity::Error,
        DiagnosticSeverity::Warning => Severity::Warning,
    };

    let mut report = MietteDiagnostic::new(&diag.message).with_severity(severity);

    if !diag.hints.is_empty() {
        report = report.with_help(diag.hints.join("\n"));
    }

    if let Some(ref loc) = diag.location {
        let label = LabeledSpan::at(loc.span_offset..loc.span_offset + loc.span_length, "here");
        report = report.with_label(label);

        let source = NamedSource::new(&loc.file, loc.source_text.clone());
        let full = miette::Report::new(report).with_source_code(source);

        let mut buf = String::new();
        if handler.render_report(&mut buf, full.as_ref()).is_ok() {
            eprint!("{buf}");
        } else {
            eprintln!("{}", diag);
        }
    } else {
        let full = miette::Report::new(report);
        let mut buf = String::new();
        if handler.render_report(&mut buf, full.as_ref()).is_ok() {
            eprint!("{buf}");
        } else {
            eprintln!("{}", diag);
        }
    }
}
