use jsonschema::{ValidationError, validate};
use serde_json::{Map, Value as JsonValue};

use crate::manifest::{ArgumentSpec, ArgumentType, CommandSpec};

impl CommandSpec {
    pub fn to_schema(&self) -> Map<String, JsonValue> {
        let mut schema = Map::new();

        schema.insert("type".to_string(), JsonValue::String("object".to_string()));

        schema.insert(
            "properties".to_string(),
            JsonValue::Object(Map::from_iter(self.args.as_ref().map_or_else(
                || Map::new(),
                |args| {
                    args.iter()
                        .map(|arg| (arg.name.clone(), JsonValue::Object(arg.to_schema())))
                        .collect::<Map<String, JsonValue>>()
                },
            ))),
        );

        schema.insert(
            "required".to_string(),
            JsonValue::Array(self.args.as_ref().map_or_else(Vec::new, |args| {
                args.iter()
                    .filter(|arg| arg.required)
                    .map(|arg| JsonValue::String(arg.name.clone()))
                    .collect()
            })),
        );

        schema
    }

    pub fn validate<'a>(&self, value: &'a JsonValue) -> Result<(), ValidationError<'a>> {
        let schema: JsonValue = self.to_schema().into();
        validate(&schema, value)
    }
}

impl ArgumentSpec {
    pub fn default_value(&self) -> Option<JsonValue> {
        if let Some(default) = &self.default {
            match &self.arg_type {
                Some(ArgumentType::String) => Some(JsonValue::String(default.clone())),
                Some(ArgumentType::Number) => default
                    .parse::<f64>()
                    .map(|n| Some(JsonValue::Number(serde_json::Number::from_f64(n).unwrap())))
                    .unwrap_or_else(|_| None),
                Some(ArgumentType::Boolean) => default
                    .parse::<bool>()
                    .map(|b| Some(JsonValue::Bool(b)))
                    .unwrap_or_else(|_| None),
                None => Some(JsonValue::String(default.clone())),
            }
        } else {
            None
        }
    }

    pub fn to_schema(&self) -> Map<String, JsonValue> {
        let mut schema = Map::new();

        schema.insert(
            "type".to_string(),
            match self.arg_type {
                Some(ArgumentType::String) => JsonValue::String("string".to_string()),
                Some(ArgumentType::Number) => JsonValue::String("number".to_string()),
                Some(ArgumentType::Boolean) => JsonValue::String("boolean".to_string()),
                None => JsonValue::String("string".to_string()),
            },
        );

        schema.insert(
            "description".to_string(),
            JsonValue::String(self.description.clone()),
        );

        if let Some(default) = &self.default_value() {
            schema.insert("default".to_string(), default.clone());
        }

        schema
    }
}
