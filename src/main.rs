use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use xml::attribute::OwnedAttribute;
use xml_utils::invert_svg;
mod colors;
mod xml_utils;

fn is_color_attribute(name: &str) -> bool {
    name == "fill" || name == "stroke"
}

fn invert_color_attributes(attributes: &[OwnedAttribute]) -> Vec<OwnedAttribute> {
    let mut inverted_attributes = Vec::with_capacity(attributes.len());
    for a in attributes.iter() {
        if is_color_attribute(&a.name.to_string()) {
            // if cannot parse the color, just use the original value
            let inverted_color =
                colors::invert_color(&a.value).unwrap_or_else(|_| a.value.to_string());
            let inverted_attribute = OwnedAttribute {
                name: a.name.clone(),
                value: inverted_color,
            };
            inverted_attributes.push(inverted_attribute);
        } else {
            inverted_attributes.push(a.clone());
        }
    }

    inverted_attributes
}

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
