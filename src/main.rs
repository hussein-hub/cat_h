use clap::Parser;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};

// Derive Parser trait to automatically parse command-line arguments
#[derive(Parser)]
// Set the program name to "cath"
#[command(name = "cath")]
// Set the program description that appears in help text
#[command(about = "A simple cat-like utility with syntax highlighting", long_about = None)]
// Define the structure that holds our command-line arguments
struct Args {
    /// Input file to read
    // Define a positional argument for the file path
    #[arg(value_name = "FILE", help = "Input file to read")]
    file_path: String,

    // Define a flag for plain output mode (short: -p, long: --plain)
    #[arg(
        short = 'p',
        long = "plain",
        help = "Output without syntax highlighting"
    )]
    plain: bool,

    #[arg(short = 'l', long = "line-numbers", help = "Show line numbers")]
    line_numbers: bool,

    #[arg(short = 's', long = "start-line", help = "Start line number")]
    start_line: Option<usize>,

    #[arg(short = 'e', long = "end-line", help = "End line number")]
    end_line: Option<usize>,
}

// Main function - entry point of the program
fn main() {
    // Parse command-line arguments into our Args struct
    let args = Args::parse();

    // Load the default syntax definitions (includes Rust, Python, JavaScript, etc.)
    let ps = SyntaxSet::load_defaults_newlines();
    // Load the default color themes (includes various dark/light themes)
    let ts = ThemeSet::load_defaults();

    // Find the appropriate syntax definition based on the file extension
    // Returns Result<Option<SyntaxReference>>, so we unwrap twice
    // If no syntax is found, fall back to plain text syntax
    let syntax = ps
        .find_syntax_for_file(&args.file_path)
        .unwrap()
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    // Create a highlighter with the detected syntax and the "base16-ocean.dark" theme
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    // Create a Path object from the file path string
    let path = Path::new(&args.file_path);
    // Read the entire file content into a String, panic with message if file can't be read
    let content = fs::read_to_string(path).expect("Failed to read the file");

    // Get a handle to stdout (standard output)
    let stdout = io::stdout();
    // Wrap stdout in a BufWriter for better performance (batches writes instead of flushing each time)
    let mut handle = BufWriter::new(stdout.lock());

    // Determine the line range to display
    let start = args.start_line.unwrap_or(1);
    let end = args.end_line.unwrap_or(usize::MAX);

    // Check if plain mode flag is set
    if args.plain {
        // In plain mode, just write the content without syntax highlighting
        if args.line_numbers {
            // With line numbers
            for (line_number, line) in LinesWithEndings::from(&content)
                .enumerate()
                .skip(start.saturating_sub(1))
                .take(end.saturating_sub(start - 1))
            {
                write!(handle, "{:4} {}", line_number + 1, line).unwrap();
            }
        } else {
            // Without line numbers
            for line in LinesWithEndings::from(&content)
                .skip(start.saturating_sub(1))
                .take(end.saturating_sub(start - 1))
            {
                write!(handle, "{}", line).unwrap();
            }
        }
        // Exit early from main function
        return;
    } else {
        // In syntax highlighting mode:
        if args.line_numbers {
            // With line numbers and syntax highlighting
            for (line_number, line) in LinesWithEndings::from(&content)
                .enumerate()
                .skip(start.saturating_sub(1))
                .take(end.saturating_sub(start - 1))
            {
                write!(handle, "{:4} ", line_number + 1).unwrap();
                // Highlight the line and get back a vector of (Style, text) pairs
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
                // Convert the styled ranges to ANSI escape codes for terminal colors
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                // Write the colored line to the buffered output
                write!(handle, "{}", escaped).unwrap();
            }
        } else {
            // Without line numbers, just syntax highlighting
            for line in LinesWithEndings::from(&content)
                .skip(start.saturating_sub(1))
                .take(end.saturating_sub(start - 1))
            {
                // Highlight the line and get back a vector of (Style, text) pairs
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
                // Convert the styled ranges to ANSI escape codes for terminal colors
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                // Write the colored line to the buffered output
                write!(handle, "{}", escaped).unwrap();
            }
        }
    }
    // BufWriter automatically flushes when it goes out of scope here
}
