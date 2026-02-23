//! TOON (Token-Oriented Object Notation) serialization
//!
//! Implements a compact, human-readable format optimized for LLM consumption.
//! TOON uses indentation-based objects and tabular arrays with explicit lengths.

use serde::Serialize;
use serde_json::Value;

/// Serialize a value to TOON format
pub fn to_toon<T: Serialize>(value: &T) -> Result<String, String> {
    let json_value = serde_json::to_value(value)
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;

    Ok(to_toon_value(&json_value, 0, false))
}

/// Convert a JSON value to TOON format
fn to_toon_value(value: &Value, depth: usize, in_array: bool) -> String {
    let indent = "  ".repeat(depth);

    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => quote_if_needed(s),
        Value::Array(arr) => {
            if arr.is_empty() {
                return "[0]:".to_string();
            }

            // Check if all elements are primitives
            let all_primitives = arr.iter().all(|v| matches!(v, Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)));

            if all_primitives {
                // Inline primitive array
                let values: Vec<String> = arr.iter()
                    .map(|v| to_toon_value(v, depth, true))
                    .collect();
                format!("[{}]: {}", arr.len(), values.join(","))
            } else if arr.iter().all(|v| matches!(v, Value::Object(_))) {
                // Check if all objects have the same keys (uniform)
                let first_keys = match arr.first() {
                    Some(Value::Object(obj)) => {
                        let mut keys: Vec<&String> = obj.keys().collect();
                        keys.sort();
                        keys
                    }
                    _ => vec![],
                };

                let is_uniform = !first_keys.is_empty() && arr.iter().all(|v| {
                    if let Value::Object(obj) = v {
                        let mut keys: Vec<&String> = obj.keys().collect();
                        keys.sort();
                        keys == first_keys
                    } else {
                        false
                    }
                });

                if is_uniform {
                    // Tabular format for uniform objects
                    let mut result = format!("[{}]{{{}}}", arr.len(), first_keys.iter().map(|k| escape_key(k)).collect::<Vec<_>>().join(","));
                    result.push_str(":\n");

                    for item in arr {
                        if let Value::Object(obj) = item {
                            let values: Vec<String> = first_keys.iter()
                                .map(|k| {
                                    obj.get(*k)
                                        .map(|v| to_toon_value(v, depth + 1, true))
                                        .unwrap_or_else(|| "null".to_string())
                                })
                                .collect();
                            result.push_str(&format!("{}{}\n", indent, values.join(",")));
                        }
                    }
                    result.trim_end().to_string()
                } else {
                    // Expanded list format for non-uniform objects
                    format_expanded_array(arr, depth)
                }
            } else {
                // Mixed types or nested arrays - expanded format
                format_expanded_array(arr, depth)
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                return if in_array { "{}".to_string() } else { "".to_string() };
            }

            let mut result = String::new();
            for (i, (key, val)) in obj.iter().enumerate() {
                if i > 0 || !in_array {
                    result.push('\n');
                }
                result.push_str(&indent);

                let key_str = escape_key(key);

                match val {
                    Value::Object(_) | Value::Array(_) => {
                        result.push_str(&key_str);
                        result.push_str(": ");
                        let nested = to_toon_value(val, depth + 1, false);
                        if matches!(val, Value::Array(_)) {
                            result.push_str(&nested);
                        } else {
                            result.push_str(&nested.trim_start());
                        }
                    }
                    _ => {
                        result.push_str(&key_str);
                        result.push_str(": ");
                        result.push_str(&to_toon_value(val, depth + 1, false));
                    }
                }
            }
            result
        }
    }
}

/// Format an array in expanded list format (- items)
fn format_expanded_array(arr: &[Value], depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let mut result = format!("[{}]:\n", arr.len());

    for item in arr {
        result.push_str(&indent);
        result.push_str("- ");
        result.push_str(&to_toon_value(item, depth + 1, true));
        result.push('\n');
    }

    result.trim_end().to_string()
}

/// Escape a key if necessary
fn escape_key(key: &str) -> String {
    // Keys that match [A-Za-z_][A-Za-z0-9_.]* don't need quotes
    if is_simple_key(key) {
        key.to_string()
    } else {
        quote_string(key)
    }
}

/// Check if a key can be unquoted
fn is_simple_key(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// Quote a string if it contains special characters or looks like a primitive
fn quote_if_needed(s: &str) -> String {
    // Always quote if empty or looks like a primitive value
    if s.is_empty() || s == "null" || s == "true" || s == "false" || s.parse::<f64>().is_ok() {
        return quote_string(s);
    }

    // Quote if contains delimiter or special characters
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') || s.contains('\t') || s.contains('\\') {
        return quote_string(s);
    }

    s.to_string()
}

/// Quote and escape a string
fn quote_string(s: &str) -> String {
    let mut result = String::from('"');
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_primitives() {
        assert_eq!(to_toon(&json!(42)).unwrap(), "42");
        assert_eq!(to_toon(&json!("hello")).unwrap(), "hello");
        assert_eq!(to_toon(&json!(true)).unwrap(), "true");
        assert_eq!(to_toon(&json!(null)).unwrap(), "null");
    }

    #[test]
    fn test_simple_object() {
        let obj = json!({"name": "Alice", "age": 30});
        let result = to_toon(&obj).unwrap();
        assert!(result.contains("name: Alice"));
        assert!(result.contains("age: 30"));
    }

    #[test]
    fn test_primitive_array() {
        let arr = json!([1, 2, 3]);
        assert_eq!(to_toon(&arr).unwrap(), "[3]: 1,2,3");
    }

    #[test]
    fn test_uniform_object_array() {
        let arr = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]);
        let result = to_toon(&arr).unwrap();
        assert!(result.contains("[2]{id,name}:"));
        assert!(result.contains("1,Alice"));
        assert!(result.contains("2,Bob"));
    }
}
