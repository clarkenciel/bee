use std::io::{BufWriter, Write as _};

use clap::Parser;

fn main() {
    let opts = Opts::parse();

    let mut stdout = BufWriter::new(std::io::stdout().lock());
    for set in opts.charsets {
        let _ = writeln!(&mut stdout, "{}: {:0>26b}", set, words::bitmask(&set));
    }
}

/// CLI to compute bitmasks for words (or sets of characters)
#[derive(Parser)]
struct Opts {
    charsets: Vec<String>,
}
