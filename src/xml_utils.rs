use std::{io::Read, io::Write};
use thiserror::Error;
use xml::{
    attribute::OwnedAttribute,
    name::{Name, OwnedName},
    reader::XmlEvent as ReaderXmlEvent,
    writer::XmlEvent as WriterXmlEvent,
    EmitterConfig, EventReader,
};

use crate::invert_color_attributes;

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

#[derive(Error, Debug)]
pub enum InvertSvgError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML read error: {0}")]
    Reader(#[from] xml::reader::Error),
    #[error("XML write error: {0}")]
    Writer(#[from] xml::writer::Error),
}

pub fn invert_svg<R: Read, W: Write>(reader: R, writer: W) -> Result<(), InvertSvgError> {
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
                    invert_color_attributes(&attributes);

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
            every_other_event => match xml_reader_event_to_xml_writer_event(&every_other_event) {
                Some(event) => xml_writer.write(event)?,
                _ => continue,
            },
        }
    }

    Ok(())
}
