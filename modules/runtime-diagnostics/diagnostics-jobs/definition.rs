use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticJob {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticJobCatalog {
    pub jobs: Vec<DiagnosticJob>,
    pub errors: Vec<String>,
    pub directory: String,
    pub selected_job_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JobFile {
    schema_version: u32,
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
}

impl TryFrom<JobFile> for DiagnosticJob {
    type Error = String;

    fn try_from(value: JobFile) -> Result<Self, Self::Error> {
        if value.schema_version != 1 {
            return Err("schemaVersion must be 1".into());
        }
        if !value.id.starts_with("user.")
            || value.id.len() <= 5
            || value.id.len() > 80
            || !value.id[5..].bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
            })
        {
            return Err("id must start with user. and contain lowercase letters, digits, dots, underscores or hyphens".into());
        }
        if value.name.trim().is_empty() || value.name.chars().count() > 80 {
            return Err("name must contain 1 to 80 characters".into());
        }
        if value.description.chars().count() > 240 {
            return Err("description must contain at most 240 characters".into());
        }
        if value.capabilities.is_empty() {
            return Err("capabilities must contain at least one supported capability".into());
        }
        let mut capabilities = value.capabilities;
        capabilities.sort();
        capabilities.dedup();
        for capability in &capabilities {
            if !matches!(
                capability.as_str(),
                "runtime" | "queue" | "mods" | "operations"
            ) {
                return Err(format!("unsupported capability: {capability}"));
            }
        }
        Ok(Self {
            id: value.id,
            name: value.name.trim().to_owned(),
            description: value.description.trim().to_owned(),
            capabilities,
        })
    }
}

pub(super) fn parse(bytes: &[u8]) -> Result<DiagnosticJob, String> {
    let definition: JobFile = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    definition.try_into()
}
