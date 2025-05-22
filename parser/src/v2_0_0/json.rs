pub struct JsonValue<T>(pub T);

impl JsonValue<String> {
    pub fn extract(
        value: &serde_json::Value,
        fields: &[&str],
    ) -> anyhow::Result<JsonValue<String>> {
        let mut ptr = value;
        for field in fields {
            ptr = &value[field];
        }

        ptr.as_str()
            .map(|s| JsonValue(s.to_string()))
            .ok_or(anyhow::anyhow!(
                "invalid str field on {value:?} @ {fields:?}"
            ))
    }
}

impl JsonValue<i64> {
    pub fn extract(value: &serde_json::Value, fields: &[&str]) -> anyhow::Result<JsonValue<i64>> {
        let mut ptr = value;
        for field in fields {
            ptr = &value[field];
        }

        ptr.as_i64().map(JsonValue).ok_or(anyhow::anyhow!(
            "invalid i64 field on {value:?} @ {fields:?}"
        ))
    }
}

impl JsonValue<serde_json::Value> {
    pub fn extract(
        value: &serde_json::Value,
        fields: &[&str],
    ) -> anyhow::Result<JsonValue<serde_json::Value>> {
        let mut ptr = value;
        for field in fields {
            ptr = &value[field];
        }

        Ok(JsonValue(ptr.clone()))
    }
}

impl JsonValue<Vec<String>> {
    pub fn extract(
        value: &serde_json::Value,
        fields: &[&str],
    ) -> anyhow::Result<JsonValue<Vec<String>>> {
        let mut ptr = value;
        for field in fields {
            ptr = &ptr[field];
        }

        ptr.as_array()
            .ok_or(anyhow::anyhow!(
                "invalid vec field on {value:?} @ {fields:?}"
            ))?
            .iter()
            .map(|s| {
                s.as_str()
                    .map(|s| s.to_string())
                    .ok_or(anyhow::anyhow!("no/invalid inner field: {s} @ {fields:?}"))
            })
            .collect::<Result<Vec<String>, _>>()
            .map(JsonValue)
    }
}
