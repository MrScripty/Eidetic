# Native runtime screenshot qualification proposal

## Additional native Bible panel capture

After the known working sample-render diagnostic, the driver clicks the actual
Bible sidebar tab through its existing native pointer route, waits for the Bible
category controls, and leaves that view untouched for ten seconds. The preserved
`eidetic-native-unedited.png` remains the script/timeline workspace. The second
allowlisted image, `eidetic-native.png`, becomes the actual Bible sidebar view,
with its hash and capture stage recorded separately. Its visible content requires
inspection and may contain an empty panel or canonical roots rather than story
entities. No sample story content is invented for the Bible, and no screenplay
editing or AI generation occurs. The app's existing sidebar may initialize its
canonical Bible roots through the normal backend command route.

The same native window ownership, timing records, PNG/log bounds, three-minute
GUI step and reviewed application-source guard remain in force. This additional
capture uses one normal job building the same pinned application source; it
does not change application code or any display/security/network settings.

## Focused render diagnostic

The proposed next run opens the real saved sample and performs no screenplay
editing. Run `37217399156` clicked the sample at `16:44:26.403Z`, captured its
fallback PNG by `16:44:27.931Z`, began editing at `16:44:28.163Z`, and failed its
typed-draft check at `16:45:57.576Z`. The PNG visibly contains the chooser; the
later accessibility snapshot contains the editor. These different observation
times do not establish persistent stale rendering or a wrong-window capture.
The previous evidence recorded PID `18990` but omitted the X11 window ID.

The workflow now selects `render-diagnostic` mode. After sample text and timeline
scene text appear in accessibility, it leaves the app untouched for ten seconds,
then captures the same verified native window by both `import -window ID` and
`import -window ID -screen`. ImageMagick documents `-screen` as reading from the
root screen instead of the selected window's drawable, while retaining the
selected window region: <https://imagemagick.org/command-line-options/#screen>.
Both PNGs retain their existing upload paths and bounds. Evidence records each
capture's exact window ID, PID, geometry, UTC start/end, read route, hash, and the
contemporaneous accessibility snapshot. Neither file is labeled an edited image;
the diagnostic status explicitly records no committed manual edit and requires
visual inspection. This does not change rendering, GPU, sandbox, or app settings.

If both delayed images show the editor, the earlier chooser image was not proof
of a persistent render failure. If only the screen-read image shows the editor,
the capture read route is implicated. If both remain on the chooser while the
contemporaneous accessibility snapshot shows the editor, visible rendering needs
further app QA. Window IDs and rechecked ownership identify selection changes.

The capture artifact uploads no binary or Cargo target. The matching application
source CI run `37205131249` has no artifacts either, so neither completed run
offers a reusable native build for this diagnostic. No further hosted build has
been started for this unpublished proposal.

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
