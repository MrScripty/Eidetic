# Native runtime screenshot qualification proposal

This isolated branch adds only a hosted capture workflow, a local sample-data
example, its desktop driver, and this note. It does not change application code.
The reviewed application source is `550650cdc794b15352013d9914eec337e5ac022d`,
tree `306d9337db0b35c485868eddb0e0f95697d5d5e6`. The job rejects application
changes outside these four qualification files before building.

## Build and execution route

The setup was checked against successful Linux CI run `37171255583`, job
`111344494613`: Ubuntu 24.04, Rust 1.92.0, GTK/WebKit dependencies,
Pumas commit `8444b50df28c3e2bd8db58fb3645fa4dd8664b27`, and normal
`ort-sys` 2.0.0-rc.12 compilation. That CI used PR6 merge checkout
`ffadc23bbb95d2379237c94f6f4fecadff85a94a`; it is setup evidence, not a
runtime screenshot or qualification of this proposal.

The proposed job builds the real desktop binary and fixture example with the
locked repository dependency route, without skipping ORT download or changing
its source. A normal Xvfb display and D-Bus session run the repository's
`./launcher.sh --run` route, including Vite for the Tauri webview. No HTTP bridge,
browser mock, JavaScript injection, sandbox override, sysctl, or container is used.

The example creates a Single-Cam project using public project services, saves it,
and seeds an imported screenplay block through the public canonical script
command. It calls no AI provider. The driver opens that saved project through
the actual splash-screen UI, uses accessibility control geometry and X11 mouse/keyboard input to
edit and save the screenplay, and requires a new committed canonical revision
plus visible edited text and timeline content before capturing the native
window. Missing accessibility, app startup, UI controls, persistence, or capture
fails the job. The picture still needs human visual inspection after execution.

## Bounds and review gate

- Trigger: push only to `ci/eidetic-runtime-capture`; no PR, release, or tag trigger.
- Before pushing, independently review all four files. Pushing starts one job.
- Permissions: `contents: read`; checkout credentials are not persisted.
- Hosted Ubuntu runner, two Cargo build jobs, no debug symbols/incremental cache.
- Resource floors: 2 GiB available memory and 6 GiB free disk before native build.
- Job limit: 30 minutes; native build step 20 minutes; capture step 3 minutes.
- Capture driver has a 110-second polling deadline and bounded subprocess waits.
- Upload allowlist: edited native PNG and unedited native-home PNG (each under 5 MiB), sanitized launch-log tail (20 KiB),
  and evidence JSON only, retained for 3 days. No DB, environment dump, raw log,
  binary, dependency tree, or credentials are uploaded.

The evidence names the application source and qualification commit, hashes the
binary and screenshot, and records the persisted before/after edit revisions.
Before navigation, the driver preserves the real native home window as
`eidetic-native-unedited.png`. Once the imported sample text is visible, the driver
updates that fallback image to the loaded native project before editing. The
evidence JSON identifies the exact fallback stage and image hash. If editing
fails, the fallback proves only that stage, with no saved manual-edit claim.
The full manual-edit proof requires a successful job and inspected edited image.
This cloud executor
cannot run the native capture: standard package installation fails on its dpkg
lock permission, WebKit/Xvfb are unavailable, and required ORT 1.24.2 is absent.

## Local preparation validation

Validate Rust formatting, Python syntax/helper behavior, workflow structure,
the qualification-only source diff, and full decision traceability. These are
proposal checks; they do not substitute for compiling or running the hosted app.
