use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use svg_invert::invert_svg;

fn main() {
    let reader = BufReader::new(stdin());
    let writer = BufWriter::new(stdout());
    match invert_svg(reader, writer) {
        Ok(_) => {
            // makes sure to flush stdout before exiting
            match stdout().flush() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
