use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue, NodeKey};
use serde_json::Value;
use std::{fmt, fs, path::Path};

#[derive(Debug)]
pub enum ConversionError {
    Io(std::io::Error),
    JsonParse(serde_json::Error),
    InvalidStructure(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::Io(err) => write!(f, "io error: {}", err),
            ConversionError::JsonParse(err) => write!(f, "json parsing error: {}", err),
            ConversionError::InvalidStructure(msg) => write!(f, "invalid json structure: {}", msg),
        }
    }
}

impl std::error::Error for ConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConversionError::Io(e) => Some(e),
            ConversionError::JsonParse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConversionError {
    fn from(err: std::io::Error) -> Self {
        ConversionError::Io(err)
    }
}

impl From<serde_json::Error> for ConversionError {
    fn from(err: serde_json::Error) -> Self {
        ConversionError::JsonParse(err)
    }
}

pub type Result<T> = std::result::Result<T, ConversionError>;

pub fn convert_file_contents(input: &Path, output: &Path, verbose: bool) -> Result<()> {
    let json_content = fs::read_to_string(input)?;
    let json_value: Value = serde_json::from_str(&json_content)?;
    let kdl_doc = json_to_kdl(json_value)?;

    // Create output directory if needed
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output, kdl_doc.to_string())?;

    if verbose {
        println!("Converted {} -> {}", input.display(), output.display());
    }

    Ok(())
}

pub fn json_to_kdl(json: Value) -> Result<KdlDocument> {
    let array = json.as_array().ok_or_else(|| {
        ConversionError::InvalidStructure("Document root must be a JSON array".to_string())
    })?;

    let mut document = KdlDocument::new();

    for value in array {
        let node = json_value_to_node(value)?;
        document.nodes_mut().push(node);
    }

    Ok(document)
}

fn json_value_to_node(value: &Value) -> Result<KdlNode> {
    let name = value
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| ConversionError::InvalidStructure("`name` must exist and be a string".to_string()))?;

    let mut node = KdlNode::new(name);

    // Handle arguments
    if let Some(arguments) = value.get("arguments") {
        let args = arguments.as_array().ok_or_else(|| {
            ConversionError::InvalidStructure("`arguments` must be an array".to_string())
        })?;

        for arg in args {
            let entry = json_value_to_entry(arg)?;
            node.push(entry);
        }
    }

    // Handle properties
    if let Some(properties) = value.get("properties") {
        let props = properties.as_object().ok_or_else(|| {
            ConversionError::InvalidStructure("`properties` must be an object".to_string())
        })?;

        for (key, prop_value) in props {
            let entry = json_value_to_entry(prop_value)?;
            node.insert(NodeKey::from(key.clone()), entry);
        }
    }

    // Handle children
    if let Some(children) = value.get("children") {
        let child_doc = json_to_kdl(children.clone())?;
        node.set_children(child_doc);
    }

    // Handle type annotation
    if let Some(type_value) = value.get("type") {
        if !type_value.is_null() {
            if let Some(type_str) = type_value.as_str() {
                node.set_ty(type_str);
            }
        }
    }

    Ok(node)
}

fn json_value_to_entry(value: &Value) -> Result<KdlEntry> {
    let (actual_value, type_annotation) = if let Some(obj) = value.as_object() {
        let val = obj.get("value").unwrap_or(value);
        let ty = obj.get("type").and_then(|t| t.as_str()).map(|s| s.to_string());
        (val, ty)
    } else {
        (value, None)
    };

    let kdl_value = match actual_value {
        Value::Null => KdlValue::Null,
        Value::Bool(b) => KdlValue::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                KdlValue::Integer(i as i128)
            } else if let Some(f) = n.as_f64() {
                KdlValue::Float(f)
            } else {
                return Err(ConversionError::InvalidStructure("invalid number value".to_string()));
            }
        }
        Value::String(s) => KdlValue::String(s.clone()),
        _ => {
            return Err(ConversionError::InvalidStructure(
                "unsupported JSON value type for KDL conversion".to_string(),
            ))
        }
    };

    let mut entry = KdlEntry::new(kdl_value);
    if let Some(ty) = type_annotation {
        entry.set_ty(ty);
    }

    Ok(entry)
}
