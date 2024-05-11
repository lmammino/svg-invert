//! # Invert SVG
//!
//! `invert_svg` is a CLI utility and a library that inverts the colors of an SVG file.
//!
//! ## Usage as a CLI utility
//!
//! ```bash
//! invert-svg < input.svg > output.svg
//! ```
//!
//! ## Usage as a library
//!
//! ```rust
//! use std::io::{stdin, stdout, BufReader, BufWriter, Write};
//! use svg_invert::invert_svg;
//!
//! let reader = BufReader::new(stdin());
//! let writer = BufWriter::new(stdout());
//! match invert_svg(reader, writer) {
//!   Ok(_) => {
//!     // makes sure to flush stdout before exiting
//!     // since we are using a BufWriter
//!     match stdout().flush() {
//!       Ok(_) => {}
//!       Err(e) => {
//!         eprintln!("Error: {e}");
//!       }
//!     }
//!   }
//!   Err(e) => {
//!     eprintln!("Error: {e}");
//!   }
//! }
//! ```
//!
//! ## Processing multiple images
//!
//! [svg_invert::invert_svg](fn.invert_svg.html) is a shortcut for creating a new [InvertSvg](struct.InvertSvg.html) instance and calling its [invert_svg](struct.InvertSvg.html#method.invert_svg) method.
//!
//! If you intend to process multiple SVG images in the same program, it might be beneficial to create a single [InvertSvg](struct.InvertSvg.html) instance and reuse it.
//!
//! In fact, an [InvertSvg](struct.InvertSvg.html) instance holds an internal cache that stores the inverted colors of the SVG images it processes.
//! So, by reusing the same instance, you might have some performance gains if your images happen to share the same colors.
//!
//! ```rust
//! use svg_invert::InvertSvg;
//!
//! let invert_svg = InvertSvg::new();
//! // call invert_svg.invert_svg multiple times
//! ```
use csscolorparser::ParseColorError;
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{Arc, RwLock},
};
use thiserror::Error;
use xml::{
    attribute::OwnedAttribute,
    name::{Name, OwnedName},
    reader::XmlEvent as ReaderXmlEvent,
    writer::XmlEvent as WriterXmlEvent,
    EmitterConfig, EventReader,
};

#[inline]
fn invert_color(color: &str) -> Result<String, ParseColorError> {
    if color == "currentColor" {
        return Ok("currentColor".to_string());
    }

    let color = csscolorparser::parse(color)?;
    let components = color.to_rgba8();
    let inv_components = [
        255 - components[0],
        255 - components[1],
        255 - components[2],
        components[3],
    ];
    let result = format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        inv_components[0], inv_components[1], inv_components[2], inv_components[3]
    );
    Ok(result)
}

#[inline]
fn is_color_attribute(name: &str) -> bool {
    name == "fill" || name == "stroke"
}

#[inline]
fn owned_name_to_name(owned: &OwnedName) -> Name<'_> {
    Name {
        local_name: owned.local_name.as_str(),
        namespace: owned.namespace.as_deref(),
        prefix: owned.prefix.as_deref(),
    }
}

fn xml_reader_event_to_xml_writer_event(
    xml_reader_event: &ReaderXmlEvent,
) -> Option<WriterXmlEvent<'_>> {
    match xml_reader_event {
        ReaderXmlEvent::StartElement {
            name,
            attributes,
            namespace,
        } => {
            let mut writer_event = WriterXmlEvent::start_element(name.local_name.as_str());
            for a in attributes.iter() {
                writer_event = writer_event.attr(owned_name_to_name(&a.name), &a.value);
            }
            for ns in namespace.iter() {
                writer_event = writer_event.ns(ns.0, ns.1);
            }
            Some(writer_event.into())
        }
        ReaderXmlEvent::EndElement { .. } => {
            let writer_event = WriterXmlEvent::end_element();
            Some(writer_event.into())
        }
        ReaderXmlEvent::Characters(data) => {
            let writer_event = WriterXmlEvent::characters(data);
            Some(writer_event)
        }
        ReaderXmlEvent::Comment(data) => {
            let writer_event = WriterXmlEvent::comment(data);
            Some(writer_event)
        }
        ReaderXmlEvent::CData(data) => {
            let writer_event = WriterXmlEvent::cdata(data);
            Some(writer_event)
        }
        ReaderXmlEvent::Whitespace(_data) => None,
        ReaderXmlEvent::ProcessingInstruction { name, data } => {
            let data = data.as_ref().map(|d| d.as_str());
            let writer_event = WriterXmlEvent::processing_instruction(name, data);
            Some(writer_event)
        }
        ReaderXmlEvent::StartDocument {
            version,
            encoding,
            standalone,
        } => {
            let writer_event = WriterXmlEvent::StartDocument {
                version: *version,
                encoding: Some(encoding.as_str()),
                standalone: *standalone,
            };
            Some(writer_event)
        }
        ReaderXmlEvent::EndDocument => None,
    }
}

/// Error type for InvertSvg.
/// It can be an IO error, an XML read error, or an XML write error.
#[derive(Error, Debug)]
pub enum InvertSvgError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML read error: {0}")]
    Reader(#[from] xml::reader::Error),
    #[error("XML write error: {0}")]
    Writer(#[from] xml::writer::Error),
}

/// Struct that inverts the colors of an SVG file.
/// It holds an internal cache that stores the inverted colors of the SVG images it processes.
/// By reusing the same instance, you might have some performance gains if your images happen to share the same colors.
#[derive(Debug, Default, Clone)]
pub struct InvertSvg {
    colors_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl InvertSvg {
    /// Creates a new InvertSvg instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inverts the colors of an SVG file.
    /// The input is a reader that reads the SVG content, and the output is a writer that is used to write the inverted SVG content.
    pub fn invert_svg<R: Read, W: Write>(
        &self,
        reader: R,
        writer: W,
    ) -> Result<(), InvertSvgError> {
        let xml_reader = EventReader::new(reader);
        let mut xml_writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(writer);

        for event in xml_reader {
            if event.is_err() {
                return Err(InvertSvgError::Reader(event.err().unwrap()));
            }

            match event.unwrap() {
                ReaderXmlEvent::StartElement {
                    name,
                    attributes,
                    namespace,
                } => {
                    let attributes_with_inverted_color: Vec<OwnedAttribute> =
                        self.invert_color_attributes(&attributes);

                    let mut writer_event = WriterXmlEvent::start_element(name.local_name.as_str());
                    for a in attributes_with_inverted_color.iter() {
                        writer_event = writer_event.attr(owned_name_to_name(&a.name), &a.value);
                    }
                    for ns in namespace.iter() {
                        writer_event = writer_event.ns(ns.0, ns.1);
                    }

                    let event: WriterXmlEvent = writer_event.into();
                    xml_writer.write(event)?;
                }
                every_other_event => match xml_reader_event_to_xml_writer_event(&every_other_event)
                {
                    Some(event) => xml_writer.write(event)?,
                    _ => continue,
                },
            }
        }

        Ok(())
    }

    fn invert_color_attributes(&self, attributes: &[OwnedAttribute]) -> Vec<OwnedAttribute> {
        let mut inverted_attributes = Vec::with_capacity(attributes.len());
        for a in attributes.iter() {
            if is_color_attribute(&a.name.to_string()) {
                // checks if we already have the current color in the cache
                let inverted_color_in_cache = {
                    let colors_cache = self.colors_cache.read().unwrap();
                    colors_cache.get(&a.value).cloned()
                };

                let inverted_color = match inverted_color_in_cache {
                    Some(inverted_color) => inverted_color,
                    None => {
                        // if cannot parse the color, just use the original value
                        let inverted_color =
                            invert_color(&a.value).unwrap_or_else(|_| a.value.to_string());
                        let mut colors_cache = self.colors_cache.write().unwrap();
                        colors_cache.insert(a.value.clone(), inverted_color.clone());
                        inverted_color
                    }
                };

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
}

/// Inverts the colors of an SVG file.
/// The input is a reader that reads the SVG content, and the output is a writer that is used to write the inverted SVG content.
/// This is a shortcut for creating a new [InvertSvg](struct.InvertSvg.html) instance and calling its [invert_svg](struct.InvertSvg.html#method.invert_svg) method.
pub fn invert_svg<R: Read, W: Write>(reader: R, writer: W) -> Result<(), InvertSvgError> {
    let invert_svg = InvertSvg::new();
    invert_svg.invert_svg(reader, writer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invert_color() {
        assert_eq!(invert_color("currentColor").unwrap(), "currentColor");
        assert_eq!(invert_color("#000000").unwrap(), "#FFFFFFFF");
        assert_eq!(invert_color("#FFFFFF").unwrap(), "#000000FF");
        assert_eq!(invert_color("#FF0000").unwrap(), "#00FFFFFF");
        assert_eq!(invert_color("#00FF00").unwrap(), "#FF00FFFF");
        assert_eq!(invert_color("#0000FF").unwrap(), "#FFFF00FF");
        assert_eq!(invert_color("#FF00FF").unwrap(), "#00FF00FF");
        assert_eq!(invert_color("#00FFFF").unwrap(), "#FF0000FF");
        assert_eq!(invert_color("#FFFF00").unwrap(), "#0000FFFF");
    }

    #[test]
    fn it_inverts_a_sample_svg() {
        let input = include_str!("../examples/some-lovely.svg");
        let expected_output = include_str!("../examples/inverted-some-lovely.svg");

        let mut output: Vec<u8> = Vec::new();
        invert_svg(input.as_bytes(), &mut output).unwrap();
        let output = String::from_utf8(output).unwrap();

        assert_eq!(output, expected_output);
    }
}
