pub fn without_credentials(xml: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use quick_xml::Reader;
    use quick_xml::Writer;
    use quick_xml::events::BytesStart;
    use quick_xml::events::Event;
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::new());
    let mut in_profile = false;
    let mut hidden_depth = 0usize;
    loop {
        let event = reader.read_event()?;
        if hidden_depth > 0 {
            match event {
                Event::Start(_) => hidden_depth += 1,
                Event::End(_) => hidden_depth -= 1,
                Event::Eof => return Err("Unclosed authentication parameter".into()),
                _ => {}
            }
            continue;
        }
        match &event {
            Event::Start(element) if element.name().as_ref() == b"profile" => in_profile = true,
            Event::End(element) if element.name().as_ref() == b"profile" => in_profile = false,
            Event::Start(element) | Event::Empty(element)
                if in_profile && element.name().as_ref() == b"parameter" =>
            {
                let attributes = element.attributes().collect::<Result<Vec<_>, _>>()?;
                let public = attributes
                    .iter()
                    .any(|a| a.key.as_ref() == b"public" && a.value.as_ref() == b"true");
                if !public {
                    let mut clean = BytesStart::new("parameter");
                    for attribute in &attributes {
                        if attribute.key.as_ref() == b"name" {
                            clean.push_attribute(attribute.clone());
                        }
                    }
                    writer.write_event(Event::Empty(clean))?;
                    if matches!(event, Event::Start(_)) {
                        hidden_depth = 1;
                    }
                    continue;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        writer.write_event(event)?;
    }
    Ok(writer.into_inner())
}
