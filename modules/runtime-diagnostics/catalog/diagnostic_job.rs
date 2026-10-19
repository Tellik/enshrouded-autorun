use serde::{Deserialize, Serialize};

pub(crate) const MAX_JOB_FILE_BYTES: usize = 64 * 1024;
pub(crate) const MAX_SCRIPT_BYTES: usize = 1024 * 1024;
static VALIDATOR: std::sync::OnceLock<Result<jsonschema::Validator, String>> =
    std::sync::OnceLock::new();

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticJob {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<DiagnosticExecution>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DiagnosticExecution {
    Lua {
        mod_id: String,
        script: String,
        #[serde(skip)]
        source: String,
    },
    ModAction {
        mod_id: String,
        action: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    Python {
        script: String,
        #[serde(skip)]
        source: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interpreter: Option<String>,
    },
    Rust {
        action: String,
    },
}

impl DiagnosticExecution {
    pub fn mod_id(&self) -> Option<&str> {
        match self {
            Self::Lua { mod_id, .. } | Self::ModAction { mod_id, .. } => Some(mod_id),
            Self::Python { .. } | Self::Rust { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticJobCatalog {
    pub jobs: Vec<DiagnosticJob>,
    pub errors: Vec<String>,
    pub directory: String,
    pub selected_job_id: String,
}

pub(super) fn parse(
    bytes: &[u8],
    load_script: impl Fn(&str) -> Result<String, String>,
) -> Result<DiagnosticJob, String> {
    if bytes.len() > MAX_JOB_FILE_BYTES {
        return Err("diagnostic job exceeds 64 KiB".into());
    }
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    let validator = VALIDATOR
        .get_or_init(|| {
            let schema: serde_json::Value =
                serde_json::from_str(include_str!("diagnostic-job.schema.json"))
                    .map_err(|error| format!("diagnostic job schema: {error}"))?;
            jsonschema::validator_for(&schema).map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)?;
    validator
        .validate(&value)
        .map_err(|error| format!("diagnostic job: {error}"))?;
    let mut job: DiagnosticJob =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    job.name = job.name.trim().to_owned();
    job.description = job.description.trim().to_owned();
    job.capabilities.sort();
    job.capabilities.dedup();
    if let Some(
        DiagnosticExecution::Lua { script, .. } | DiagnosticExecution::Python { script, .. },
    ) = &job.execution
    {
        if std::path::Path::new(script.as_str())
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("diagnostic script must be a relative path beneath the jobs folder".into());
        }
        let loaded = load_script(script)?;
        if loaded.len() > MAX_SCRIPT_BYTES {
            return Err("diagnostic script exceeds 1 MiB".into());
        }
        if let Some(
            DiagnosticExecution::Lua { source, .. } | DiagnosticExecution::Python { source, .. },
        ) = &mut job.execution
        {
            *source = loaded;
        }
    }
    Ok(job)
}
