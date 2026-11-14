//! The tool commands the palettes queue (D-018). Palettes edit a copy of a sub tool; the
//! difference to the mirrored state becomes `subtool.*` commands.

use brushye_command::commands::{self as c, Command};
use brushye_command::tools::Target;
use serde_json::{Map, Value as Json, json};

use super::tools::{SubTool, ToolId};

fn at(t: Target) -> (Option<ToolId>, Option<u32>, Option<u32>) {
    (Some(t.tool), Some(t.group as u32), Some(t.item as u32))
}

pub fn select(t: Target) -> Command {
    let (tool, group, item) = at(t);
    Command::SubtoolSelect(c::SubtoolSelect { tool, group, item })
}

/// Select a group tab (its selected sub tool).
pub fn select_group(tool: ToolId, group: usize) -> Command {
    Command::SubtoolSelect(c::SubtoolSelect {
        tool: Some(tool),
        group: Some(group as u32),
        item: None,
    })
}

pub fn copy(t: Target, name: Option<String>) -> Command {
    let (tool, group, item) = at(t);
    Command::SubtoolCopy(c::SubtoolCopy {
        tool,
        group,
        item,
        name,
    })
}

pub fn delete(t: Target) -> Command {
    let (tool, group, item) = at(t);
    Command::SubtoolDelete(c::SubtoolDelete { tool, group, item })
}

pub fn move_to(t: Target, to: usize) -> Command {
    let (tool, group, item) = at(t);
    Command::SubtoolMove(c::SubtoolMove {
        tool,
        group,
        item,
        to: to as u32,
    })
}

pub fn lock(t: Target, locked: bool) -> Command {
    let (tool, group, item) = at(t);
    Command::SubtoolLock(c::SubtoolLock {
        tool,
        group,
        item,
        locked,
    })
}

pub fn reset(t: Target) -> Command {
    let (tool, group, item) = at(t);
    Command::SubtoolReset(c::SubtoolReset { tool, group, item })
}

pub fn save_default(t: Target) -> Command {
    let (tool, group, item) = at(t);
    Command::SubtoolSaveDefault(c::SubtoolSaveDefault { tool, group, item })
}

pub fn add_defaults(tool: ToolId) -> Command {
    Command::SubtoolAddDefaults(c::SubtoolAddDefaults { tool: Some(tool) })
}

pub fn add(tool: ToolId, name: &str, settings: Option<String>) -> Command {
    Command::SubtoolAdd(c::SubtoolAdd {
        tool: Some(tool),
        group: None,
        name: name.to_owned(),
        settings,
    })
}

pub fn rename_group(tool: ToolId, group: usize, name: &str) -> Command {
    Command::SubtoolGroupSet(c::SubtoolGroupSet {
        group: group as u32,
        tool: Some(tool),
        name: name.to_owned(),
    })
}

/// `subtool.set` for a changed name and/or settings; `None` when nothing changed.
fn set(t: Target, old: &SubTool, new: &SubTool) -> Option<Command> {
    let (o, n) = (old.preset.to_json(), new.preset.to_json());
    let mut args: Map<String, Json> = n
        .iter()
        .filter(|(k, v)| o.get(*k) != Some(*v))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let clear: Vec<&str> = o
        .keys()
        .filter(|k| !n.contains_key(*k))
        .map(String::as_str)
        .collect();
    if !clear.is_empty() {
        args.insert("clear".into(), json!(clear.join(",")));
    }
    if old.name != new.name && !new.name.trim().is_empty() {
        args.insert("name".into(), json!(new.name));
    }
    if args.is_empty() {
        return None;
    }
    args.insert("tool".into(), json!(t.tool.name()));
    args.insert("group".into(), json!(t.group));
    args.insert("item".into(), json!(t.item));
    Command::from_json(&json!({ "cmd": "subtool.set", "v": 1, "args": args })).ok()
}

/// Commands that turn sub tool `old` (at `t`) into `new`: name, settings, rows and lock.
pub fn changes(t: Target, old: &SubTool, new: &SubTool) -> Vec<Command> {
    let mut out = Vec::new();
    if new.locked() != old.locked() {
        out.push(lock(t, new.locked()));
    }
    // Invalid settings (mid-edit) aren't sent; the editor shows the error.
    if new.preset.validate().is_ok()
        && let Some(cmd) = set(t, old, new)
    {
        out.push(cmd);
    }
    let (tool, group, item) = at(t);
    for k in &new.shown {
        if !old.shown.contains(k) {
            out.push(Command::SubtoolShowRow(c::SubtoolShowRow {
                tool,
                group,
                item,
                row: k.clone(),
                shown: true,
            }));
        }
    }
    for k in &old.shown {
        if !new.shown.contains(k) {
            out.push(Command::SubtoolShowRow(c::SubtoolShowRow {
                tool,
                group,
                item,
                row: k.clone(),
                shown: false,
            }));
        }
    }
    out
}

pub fn color_set(chip: brushye_command::tools::Chip, rgb: [u8; 3]) -> Command {
    Command::ColorSet(c::ColorSet {
        chip,
        color: brushye_command::core::Rgba8([rgb[0], rgb[1], rgb[2], 255]),
    })
}

pub fn color_active(chip: brushye_command::tools::Chip) -> Command {
    Command::ColorActive(c::ColorActive { chip })
}

#[cfg(test)]
mod tests {
    use super::*;
    use brushye_command::Session;

    /// Edits made through `changes` reproduce the edited sub tool in the session.
    #[test]
    fn changes_reproduce_the_edit() {
        let mut s = Session::new();
        let t = s
            .tools()
            .target(None, None, None)
            .unwrap_or_else(|e| unreachable!("{e}"));
        let old = s.tools().get(t).clone();
        let mut new = old.clone();
        new.name = "Edited".into();
        new.preset.size = 33.0;
        new.preset.size_pressure = None;
        new.preset.hardness = Some(2);
        new.shown.retain(|k| k != "aa");
        new.shown.push("hardness".into());
        new.base = Some(new.preset.clone());
        for cmd in changes(t, &old, &new) {
            if let Err(e) = s.execute(cmd) {
                unreachable!("{e}");
            }
        }
        let got = s.tools().get(t);
        assert_eq!(got.name, "Edited");
        assert_eq!(got.preset, new.preset);
        assert_eq!(got.shown, new.shown);
        assert!(got.locked());
        assert!(changes(t, got, got).is_empty());
    }
}
