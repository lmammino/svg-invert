use csscolorparser::ParseColorError;

pub fn invert_color(color: &str) -> Result<String, ParseColorError> {
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
}
