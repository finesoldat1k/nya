//! Diagnostics and error reporting for nya

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A source code location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub file_id: usize,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(file_id: usize, start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { file_id, start, end, line, column }
    }
    
    pub fn dummy() -> Self {
        Self { file_id: 0, start: 0, end: 0, line: 1, column: 1 }
    }
    
    pub fn merge(self, other: Span) -> Span {
        Span {
            file_id: self.file_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: self.column.min(other.column),
        }
    }
}

/// A labeled span for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub style: LabelStyle,
}

impl Label {
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Primary,
        }
    }
    
    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Secondary,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Note,
    Help,
}

/// A diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }
    
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }
    
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }
    
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
    
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }
    
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
    
    pub fn label(span: Span, message: impl Into<String>) -> Label {
        Label {
            span,
            message: message.into(),
            style: LabelStyle::Primary,
        }
    }
    
    pub fn secondary_label(span: Span, message: impl Into<String>) -> Label {
        Label {
            span,
            message: message.into(),
            style: LabelStyle::Secondary,
        }
    }
}

/// Collection of diagnostics
#[derive(Debug, Default, Clone)]
pub struct DiagnosticBag {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
    
    pub fn extend(&mut self, other: DiagnosticBag) {
        self.diagnostics.extend(other.diagnostics);
    }
    
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }
    
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error).collect()
    }
    
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).collect()
    }
}

/// Source file for error reporting
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: usize,
    pub name: String,
    pub content: String,
    pub lines: Vec<usize>, // byte offsets of line starts
}

impl SourceFile {
    pub fn new(id: usize, name: String, content: String) -> Self {
        let mut lines = vec![0];
        for (i, c) in content.char_indices() {
            if c == '\n' {
                lines.push(i + 1);
            }
        }
        Self { id, name, content, lines }
    }
    
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.lines.binary_search(&offset).unwrap_or_else(|i| i.saturating_sub(1));
        let line_start = self.lines[line];
        let column = offset.saturating_sub(line_start) + 1;
        (line + 1, column)
    }
    
    pub fn get_line(&self, line: usize) -> Option<&str> {
        let line_idx = line.checked_sub(1)?;
        let start = self.lines.get(line_idx)?;
        let end = self.lines.get(line_idx + 1).map(|&o| o.saturating_sub(1)).unwrap_or(self.content.len());
        self.content.get(*start..end)
    }
}

/// Source map for managing multiple files
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_file(&mut self, name: String, content: String) -> usize {
        let id = self.files.len();
        self.files.push(SourceFile::new(id, name, content));
        id
    }
    
    pub fn get_file(&self, id: usize) -> Option<&SourceFile> {
        self.files.get(id)
    }
}

/// Emit diagnostics to stderr with colors
pub fn emit_diagnostics(diagnostics: &[Diagnostic], source_map: &SourceMap) -> Result<()> {
    use colored::*;
    
    for diag in diagnostics {
        let severity_str = match diag.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Info => "info".blue().bold(),
            Severity::Note => "note".cyan().bold(),
            Severity::Help => "help".green().bold(),
        };
        
        if let Some(label) = diag.labels.first() {
            if let Some(file) = source_map.get_file(label.span.file_id) {
                let (line, col) = file.line_col(label.span.start);
                eprintln!("{}:{}:{}: {}: {}", file.name, line, col, severity_str, diag.message);
                
                if let Some(source_line) = file.get_line(line) {
                    eprintln!("{}", source_line);
                    let indent = " ".repeat(col - 1);
                    let pointer = "^".repeat((label.span.end - label.span.start).max(1));
                    eprintln!("{}{}{}", indent, pointer.bright_red(), "");
                }
            }
        } else {
            eprintln!("{}: {}", severity_str, diag.message);
        }
        
        if let Some(code) = &diag.code {
            eprintln!("  {}", format!("---> [{}]", code).dimmed());
        }
        
        for note in &diag.notes {
            eprintln!("  {}: {}", "note".cyan().bold(), note);
        }
        
        for label in &diag.labels[1..] {
            if let Some(file) = source_map.get_file(label.span.file_id) {
                let (line, col) = file.line_col(label.span.start);
                let style = match label.style {
                    LabelStyle::Primary => "━━",
                    LabelStyle::Secondary => "···",
                };
                let style_colored = match label.style {
                    LabelStyle::Primary => style.red(),
                    LabelStyle::Secondary => style.blue(),
                };
                eprintln!("  {} {}:{}: {} {}", style_colored, file.name, line, col, label.message);
            }
        }
    }
    
    Ok(())
}