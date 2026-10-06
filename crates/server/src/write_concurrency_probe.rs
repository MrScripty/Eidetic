//! Test-only, identity-scoped barriers for actual public command/autosave ordering.
use eidetic_core::contracts::CommandId;
use parking_lot::Mutex;
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub(crate) enum CommandStage {
    SignatureRead,
    FirstInsert { extended_error_code: Option<i32> },
}
pub(crate) struct CommandProbe {
    pub stages: UnboundedSender<CommandStage>,
    pub release: std::sync::mpsc::Receiver<()>,
}
#[derive(Debug)]
pub(crate) enum AutosaveStage {
    BeforePersistence,
    Persisted(Result<(), String>),
}
fn commands() -> &'static Mutex<HashMap<CommandId, CommandProbe>> {
    static PROBES: OnceLock<Mutex<HashMap<CommandId, CommandProbe>>> = OnceLock::new();
    PROBES.get_or_init(Default::default)
}
fn autosaves() -> &'static Mutex<HashMap<PathBuf, UnboundedSender<AutosaveStage>>> {
    static PROBES: OnceLock<Mutex<HashMap<PathBuf, UnboundedSender<AutosaveStage>>>> =
        OnceLock::new();
    PROBES.get_or_init(Default::default)
}
pub(crate) enum Registration {
    Command(CommandId),
    Autosave(PathBuf),
}
impl Drop for Registration {
    fn drop(&mut self) {
        match self {
            Self::Command(id) => {
                commands().lock().remove(id);
            }
            Self::Autosave(path) => {
                autosaves().lock().remove(path);
            }
        }
    }
}
pub(crate) fn command(id: CommandId, probe: CommandProbe) -> Registration {
    assert!(commands().lock().insert(id, probe).is_none());
    Registration::Command(id)
}
pub(crate) fn autosave(path: PathBuf, sender: UnboundedSender<AutosaveStage>) -> Registration {
    assert!(autosaves().lock().insert(path.clone(), sender).is_none());
    Registration::Autosave(path)
}
pub(crate) fn take_command(id: CommandId) -> Option<CommandProbe> {
    commands().lock().remove(&id)
}
pub(crate) fn observe_autosave(path: &std::path::Path, stage: AutosaveStage) {
    if let Some(sender) = autosaves().lock().get(path).cloned() {
        let _ = sender.send(stage);
    }
}
