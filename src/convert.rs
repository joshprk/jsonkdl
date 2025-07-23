use kdl::{KdlDocument, KdlEntry, KdlIdentifier, KdlNode, KdlValue};
use serde_json::Value as JsonValue;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum KdlVersion {
    V1,
    #[default]
    V2,
}

pub fn convert_file_content(input: &Path, version: KdlVersion) -> Result<String> {
    let json_content = fs::read_to_string(input)?;
    let json_value: JsonValue = serde_json::from_str(&json_content)?;

    let mut document = convert_document(&json_value)?;

    // For some reason, you MUST autoformat before ensuring version.
    document.autoformat();

    match version {
        KdlVersion::V1 => document.ensure_v1(),
        KdlVersion::V2 => document.ensure_v2(),
    }

    Ok(document.to_string())
}

pub fn convert_and_write_file_content(
    input: &Path,
    output: &Path,
    verbose: bool,
    version: KdlVersion,
) -> Result<()> {
    let kdl_doc_content = convert_file_content(input, version)?;

    fs::write(output, kdl_doc_content)?;

    if verbose {
        println!("converted {} -> {}", input.display(), output.display());
    }

    Ok(())
}

pub fn convert_document(json: &JsonValue) -> Result<KdlDocument> {
    let json = json.as_array().ok_or_else(|| {
        ConversionError::InvalidStructure("document root must be an array".to_string())
    })?;

    let mut document = KdlDocument::new();

    for value in json {
        let node = convert_node(value)?;
        document.nodes_mut().push(node);
    }

    Ok(document)
}

fn convert_node(json: &JsonValue) -> Result<KdlNode> {
    let json = json
        .as_object()
        .ok_or_else(|| ConversionError::InvalidStructure("node must be an object".to_string()))?;

    let name = match json.get("name") {
        Some(JsonValue::String(name)) => Ok(name.as_str()),
        Some(_) => Err(ConversionError::InvalidStructure(
            "name must be a string".to_string(),
        )),
        None => Err(ConversionError::InvalidStructure(
            "node must have a name".to_string(),
        )),
    }?;

    let mut node = KdlNode::new(name);

    // Handle arguments
    if let Some(arguments) = json.get("arguments") {
        let arguments = arguments.as_array().ok_or_else(|| {
            ConversionError::InvalidStructure("arguments must be an array".to_string())
        })?;

        for arg in arguments {
            let entry = convert_entry(arg)?;
            node.push(entry);
        }
    }

    // Handle properties
    if let Some(properties) = json.get("properties") {
        let properties = properties.as_object().ok_or_else(|| {
            ConversionError::InvalidStructure("properties must be an object".to_string())
        })?;

        for (key, prop_value) in properties {
            let mut entry = convert_entry(prop_value)?;
            entry.set_name(Some(key.as_str()));
            node.push(entry);
        }
    }

    // Handle children
    if let Some(children) = json.get("children") {
        let children = convert_document(children)?;
        node.set_children(children);
    }

    // Handle type annotation
    if let Some(ty) = json.get("type") {
        if let Some(ty) = convert_type(ty)? {
            node.set_ty(ty);
        }
    }

    Ok(node)
}

fn convert_entry(json: &JsonValue) -> Result<KdlEntry> {
    let value = convert_value(json.get("value").unwrap_or(json))?;

    let mut entry = KdlEntry::new(value);

    if let Some(ty) = json.get("type") {
        if let Some(ty) = convert_type(ty)? {
            entry.set_ty(ty);
        }
    }

    Ok(entry)
}

fn convert_value(json: &JsonValue) -> Result<KdlValue> {
    match json {
        JsonValue::Null => Ok(KdlValue::Null),
        JsonValue::Bool(b) => Ok(KdlValue::Bool(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(KdlValue::Integer(i as i128))
            } else if let Some(f) = n.as_f64() {
                Ok(KdlValue::Float(f))
            } else {
                Err(ConversionError::InvalidStructure(
                    "invalid number value".to_string(),
                ))
            }
        }
        JsonValue::String(s) => Ok(KdlValue::String(s.clone())),
        _ => Err(ConversionError::InvalidStructure(
            "unsupported json value type for kdl conversion".to_string(),
        )),
    }
}

fn convert_type(json: &JsonValue) -> Result<Option<KdlIdentifier>> {
    match json {
        JsonValue::String(ty) => Ok(Some(KdlIdentifier::from(ty.as_str()))),
        JsonValue::Null => Ok(None),
        _ => Err(ConversionError::InvalidStructure(
            "type must be a string or null".to_string(),
        )),
    }
}
