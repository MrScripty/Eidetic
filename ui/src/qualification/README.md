# Manual fact native qualification controls

This QA branch is based on exact `6e3bad3cdd780e8a66bcaffabb7baf6ff6c782f0`.
Its Vite config adds visibly labelled controls only in the native qualifier build.
Product files and Tauri commands remain unchanged. Wrappers forward public
analysis/decision calls unchanged and retain exact IDs for explicit replay.
Additional controls edit/restore an existing association with public commands
and read canonical impacts. No database writes, DOM/IPC injection, model download,
real inference or automatic product actions are introduced. Ordinary native
screenplay Save, consumed-field selection, Analyze, Reject and Accept execute.
