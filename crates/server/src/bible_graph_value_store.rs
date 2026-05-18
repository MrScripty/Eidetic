use eidetic_core::contracts::{FieldValue, ObjectKind};

use crate::history_store::HistoryStoreError;

#[derive(Debug)]
pub(crate) struct SqlGraphFieldValue {
    pub(crate) value_type: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) integer: Option<i64>,
    pub(crate) number: Option<f64>,
    pub(crate) bool_value: Option<i64>,
    pub(crate) ref_kind: Option<String>,
    pub(crate) ref_id: Option<String>,
    pub(crate) asset_ref: Option<String>,
}

impl SqlGraphFieldValue {
    pub(crate) fn none() -> Self {
        Self {
            value_type: None,
            text: None,
            integer: None,
            number: None,
            bool_value: None,
            ref_kind: None,
            ref_id: None,
            asset_ref: None,
        }
    }

    pub(crate) fn from_field_value(value: Option<&FieldValue>) -> Result<Self, HistoryStoreError> {
        let Some(value) = value else {
            return Ok(Self::none());
        };

        let mut stored = Self::none();
        match value {
            FieldValue::Text(value) => {
                stored.value_type = Some("text".to_string());
                stored.text = Some(value.clone());
            }
            FieldValue::Integer(value) => {
                stored.value_type = Some("integer".to_string());
                stored.integer = Some(*value);
            }
            FieldValue::Number(value) => {
                stored.value_type = Some("number".to_string());
                stored.number = Some(*value);
            }
            FieldValue::Bool(value) => {
                stored.value_type = Some("bool".to_string());
                stored.bool_value = Some(if *value { 1 } else { 0 });
            }
            FieldValue::ObjectRef { kind, id } => {
                stored.value_type = Some("object_ref".to_string());
                stored.ref_kind = Some(encode_object_kind(kind)?);
                stored.ref_id = Some(id.clone());
            }
            FieldValue::AssetRef(value) => {
                stored.value_type = Some("asset_ref".to_string());
                stored.asset_ref = Some(value.clone());
            }
        }
        Ok(stored)
    }

    pub(crate) fn into_field_value(self) -> Result<Option<FieldValue>, HistoryStoreError> {
        let Some(value_type) = self.value_type else {
            return Ok(None);
        };

        match value_type.as_str() {
            "text" => Ok(Some(FieldValue::Text(required(self.text, "text")?))),
            "integer" => Ok(Some(FieldValue::Integer(required(
                self.integer,
                "integer",
            )?))),
            "number" => Ok(Some(FieldValue::Number(required(self.number, "number")?))),
            "bool" => Ok(Some(FieldValue::Bool(
                required(self.bool_value, "bool")? != 0,
            ))),
            "object_ref" => Ok(Some(FieldValue::ObjectRef {
                kind: decode_object_kind(&required(self.ref_kind, "ref_kind")?)?,
                id: required(self.ref_id, "ref_id")?,
            })),
            "asset_ref" => Ok(Some(FieldValue::AssetRef(required(
                self.asset_ref,
                "asset_ref",
            )?))),
            other => Err(HistoryStoreError::InvalidValue(format!(
                "unknown graph field value type: {other}"
            ))),
        }
    }
}

fn required<T>(value: Option<T>, field_name: &'static str) -> Result<T, HistoryStoreError> {
    value.ok_or(HistoryStoreError::MissingColumn(field_name))
}

fn encode_object_kind(value: &ObjectKind) -> Result<String, HistoryStoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected object kind to serialize as string".to_string(),
        )),
    }
}

fn decode_object_kind(value: &str) -> Result<ObjectKind, HistoryStoreError> {
    Ok(serde_json::from_value(serde_json::Value::String(
        value.to_string(),
    ))?)
}
