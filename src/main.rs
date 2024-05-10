use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use xml::{
    attribute::OwnedAttribute,
    name::{Name, OwnedName},
    reader::XmlEvent as ReaderXmlEvent,
    writer::{EmitterConfig, XmlEvent as WriterXmlEvent},
    EventReader,
};

fn is_color_attribute(name: &str) -> bool {
    name == "fill" || name == "stroke"
}

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

fn invert_color_attributes(attributes: &[OwnedAttribute]) -> Vec<OwnedAttribute> {
    let mut inverted_attributes = Vec::with_capacity(attributes.len());
    for a in attributes.iter() {
        if is_color_attribute(&a.name.to_string()) {
            // TODO: implement missing function
            inverted_attributes.push(invert_color_attribute(a));
        } else {
            inverted_attributes.push(a.clone());
        }
    }

    inverted_attributes
}

fn main() {
    let reader = BufReader::new(stdin());
    let xml_reader = EventReader::new(reader);
    let writer = BufWriter::new(stdout());
    let mut xml_writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(writer);

    for e in xml_reader {
        match e {
            Ok(ReaderXmlEvent::StartElement {
                name,
                attributes,
                namespace,
            }) => {
                let attributes_with_inverted_color: Vec<OwnedAttribute> = attributes
                    .iter()
                    .cloned()
                    .map(|a| {
                        if is_color_attribute(&a.name.to_string()) {
                            return invert_color_attribute(&a);
                        }
                        a
                    })
                    .collect();

                let mut writer_event = WriterXmlEvent::start_element(name.local_name.as_str());
                for a in attributes_with_inverted_color.iter() {
                    writer_event = writer_event.attr(owned_name_to_name(&a.name), &a.value);
                }
                for ns in namespace.iter() {
                    writer_event = writer_event.ns(ns.0, ns.1);
                }

                let event: WriterXmlEvent = writer_event.into();
                match xml_writer.write(event) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            every_other_event => {
                match xml_reader_event_to_xml_writer_event(&every_other_event.unwrap()) {
                    Some(event) => match xml_writer.write(event) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error: {e}");
                            break;
                        }
                    },
                    None => {
                        continue;
                    }
                }
            }
        }
    }

    // makes sure to flush stdout before exiting
    match stdout().flush() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
