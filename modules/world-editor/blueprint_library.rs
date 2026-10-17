use crate::panel_layout::DEFAULT_HEIGHT;
use crate::panel_layout::DEFAULT_WIDTH;
use crate::panel_layout::MAX_HEIGHT;
use crate::panel_layout::MAX_POSITION;
use crate::panel_layout::MAX_WIDTH;
use crate::panel_layout::MIN_HEIGHT;
use crate::panel_layout::MIN_WIDTH;
use crate::panel_layout::TOP_OFFSET;
use base64::Engine as _;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlueprintCard {
    pub(crate) name: String,
    pub(crate) image: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ViewState {
    pub(crate) blueprints: Vec<BlueprintCard>,
    pub(crate) process_state: String,
    pub(crate) process_operation: String,
    pub(crate) process_phase: String,
    pub(crate) process_completed: u8,
    pub(crate) process_total: u8,
    pub(crate) process_sequence: u64,
    pub(crate) stage: String,
    pub(crate) hint: String,
    pub(crate) selected: String,
    pub(crate) axis: String,
    pub(crate) turns: u8,
    pub(crate) panel_width: u32,
    pub(crate) panel_height: u32,
    pub(crate) panel_position_x: i32,
    pub(crate) panel_position_y: u32,
}

pub(crate) fn view_state(root: &Path) -> Result<ViewState, Box<dyn std::error::Error>> {
    let exports = shroudforge_loader_config::paths::export_dir(root);
    let directory = exports.join("world-editor/blueprints");
    let mut process_state = "idle".to_owned();
    let mut process_operation = String::new();
    let mut process_phase = String::new();
    let mut process_completed = 0;
    let mut process_total = 0;
    let mut process_sequence = 0;
    let mut stage = "idle".to_owned();
    let mut hint =
        "Select the Building Hammer, choose a Single Voxel, aim at the first corner, then press F5."
            .to_owned();
    let mut selected = String::new();
    let mut axis = "y".to_owned();
    let mut turns = 0;
    let mut panel_width = DEFAULT_WIDTH;
    let mut panel_height = DEFAULT_HEIGHT;
    let mut panel_position_x = -1;
    let mut panel_position_y = TOP_OFFSET as u32;
    if let Ok(contents) = fs::read_to_string(exports.join("world-editor/editor-state.txt")) {
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("processState=") {
                process_state = value.to_owned();
            } else if let Some(value) = line.strip_prefix("processOperation=") {
                process_operation = value.to_owned();
            } else if let Some(value) = line.strip_prefix("processPhase=") {
                process_phase = value.to_owned();
            } else if let Some(value) = line.strip_prefix("processCompleted=") {
                process_completed = value.parse::<u8>().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("processTotal=") {
                process_total = value.parse::<u8>().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("processSequence=") {
                process_sequence = value.parse::<u64>().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("stage=") {
                stage = value.to_owned();
            } else if let Some(value) = line.strip_prefix("hint=") {
                hint = value.to_owned();
            } else if let Some(value) = line.strip_prefix("selected=") {
                selected = value.to_owned();
            } else if let Some(value) = line.strip_prefix("axis=") {
                axis = value.to_owned();
            } else if let Some(value) = line.strip_prefix("turns=") {
                turns = value.parse::<u8>().unwrap_or(0) % 4;
            } else if let Some(value) = line.strip_prefix("panelWidth=") {
                panel_width = value
                    .parse::<u32>()
                    .unwrap_or(DEFAULT_WIDTH)
                    .clamp(MIN_WIDTH, MAX_WIDTH);
            } else if let Some(value) = line.strip_prefix("panelHeight=") {
                panel_height = value
                    .parse::<u32>()
                    .unwrap_or(DEFAULT_HEIGHT)
                    .clamp(MIN_HEIGHT, MAX_HEIGHT);
            } else if let Some(value) = line.strip_prefix("panelPositionX=") {
                panel_position_x = value.parse::<i32>().unwrap_or(-1).clamp(-1, MAX_POSITION);
            } else if let Some(value) = line.strip_prefix("panelPositionY=") {
                panel_position_y = value
                    .parse::<u32>()
                    .unwrap_or(TOP_OFFSET as u32)
                    .min(MAX_POSITION as u32);
            }
        }
    }
    let mut blueprints = Vec::new();
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("sfbp"))
            {
                let Some(name) = path.file_stem().and_then(|value| value.to_str()) else {
                    continue;
                };
                if !valid_blueprint_name(name) {
                    continue;
                }
                let image_path = path.with_file_name(format!("{name}.thumb.png"));
                let image_path = if image_path.is_file() {
                    image_path
                } else {
                    path.with_extension("png")
                };
                let image = fs::read(image_path).ok().map(|bytes| {
                    format!(
                        "data:image/png;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(bytes)
                    )
                });
                blueprints.push(BlueprintCard {
                    name: name.to_owned(),
                    image,
                });
            }
        }
    }
    blueprints.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(ViewState {
        process_state,
        process_operation,
        process_phase,
        process_completed,
        process_total,
        process_sequence,
        blueprints,
        stage,
        hint,
        selected,
        axis,
        turns,
        panel_width,
        panel_height,
        panel_position_x,
        panel_position_y,
    })
}

pub(crate) fn empty_state() -> ViewState {
    ViewState {
        blueprints: Vec::new(),
        process_state: "idle".into(),
        process_operation: String::new(),
        process_phase: String::new(),
        process_completed: 0,
        process_total: 0,
        process_sequence: 0,
        stage: "idle".into(),
        hint: "Select the Building Hammer, choose a Single Voxel, aim at the first corner, then press F5."
            .into(),
        selected: String::new(),
        axis: "y".into(),
        turns: 0,
        panel_width: DEFAULT_WIDTH,
        panel_height: DEFAULT_HEIGHT,
        panel_position_x: -1,
        panel_position_y: TOP_OFFSET as u32,
    }
}

pub(crate) fn valid_blueprint_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}
