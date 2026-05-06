use std::io::BufRead;
use std::path::PathBuf;

const TOC_HEADER: &str = "// Table of Contents\n// ==================\n";
const TOC_FOOTER: &str = "// ==================\n";

pub struct TableOfContents;

impl TableOfContents {
    pub fn generate(files: &[(PathBuf, Vec<u8>)], header_lines: usize) -> Option<Vec<u8>> {
        if files.is_empty() {
            return None;
        }

        let mut toc = String::from(TOC_HEADER);
        let mut current_line = header_lines + 2; // +2 for TOC header lines

        for (path, content) in files {
            let line_count = content.lines().count();
            toc.push_str(&format!("// Ln{:<6}: {}\n", current_line, path.display()));
            current_line += line_count + 1; // +1 for separator line
        }

        toc.push_str(TOC_FOOTER);
        Some(toc.into_bytes())
    }

    pub fn count_header_lines(head_content: Option<&[u8]>) -> usize {
        head_content.map(|bytes| bytes.lines().count()).unwrap_or(0)
    }
}
