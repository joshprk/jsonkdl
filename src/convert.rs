use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue, NodeKey};
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
        ConversionError::InvalidStructure("document root must be json array".to_string())
    })?;

    let mut document = KdlDocument::new();

    for value in json {
        let node = convert_node(value)?;
        document.nodes_mut().push(node);
    }

    Ok(document)
}

fn convert_node(json: &JsonValue) -> Result<KdlNode> {
    let name = json.get("name").and_then(|n| n.as_str()).ok_or_else(|| {
        ConversionError::InvalidStructure("name must be non-empty string".to_string())
    })?;

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
            let entry = convert_entry(prop_value)?;
            node.insert(NodeKey::from(key.clone()), entry);
        }
    }

    // Handle children
    if let Some(children) = json.get("children") {
        let children = convert_document(children)?;
        node.set_children(children);
    }

    // Handle type annotation
    if let Some(type_value) = json.get("type") {
        if !type_value.is_null() {
            if let Some(type_str) = type_value.as_str() {
                node.set_ty(type_str);
            }
        }
    }

    Ok(node)
}

fn convert_entry(json: &JsonValue) -> Result<KdlEntry> {
    let (actual_value, type_annotation) = if let Some(obj) = json.as_object() {
        let val = obj.get("value").unwrap_or(json);
        let ty = obj
            .get("type")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());
        (val, ty)
    } else {
        (json, None)
    };

    let kdl_value = match actual_value {
        JsonValue::Null => KdlValue::Null,
        JsonValue::Bool(b) => KdlValue::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                KdlValue::Integer(i as i128)
            } else if let Some(f) = n.as_f64() {
                KdlValue::Float(f)
            } else {
                return Err(ConversionError::InvalidStructure(
                    "invalid number value".to_string(),
                ));
            }
        }
        JsonValue::String(s) => KdlValue::String(s.clone()),
        _ => {
            return Err(ConversionError::InvalidStructure(
                "unsupported json value type for kdl conversion".to_string(),
            ));
        }
    };

    let mut entry = KdlEntry::new(kdl_value);
    if let Some(ty) = type_annotation {
        entry.set_ty(ty);
    }

    Ok(entry)
}
