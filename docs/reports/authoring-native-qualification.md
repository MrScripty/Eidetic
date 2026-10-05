# Integrated native screenplay authoring qualification

The frozen application source is `72ae4806ef70a8465e6fe26f5ce8e891b0dfb8f1`.
This isolated qualification adds a hosted workflow, public-service empty-project
fixture, and native accessibility/X11 driver. The workflow verifies its ancestry
and rejects application changes outside its five qualification files.

Ubuntu 24.04 installs standard GTK/WebKit/Xvfb dependencies and builds the locked
native desktop and fixture example through the normal pinned Pumas/ORT route.
It runs the real server library tests and launches the actual Tauri application
through `launcher.sh --run`. There is no skipped dependency download, injected
JavaScript, substituted IPC, network-policy change or sandbox override.

The native flow creates the first screenplay in an empty canonical document,
keeps the exact creation draft through Graph/Split navigation and return to Script,
and saves it. Both Graph and Split remove the screenplay panel; retained text is
checked after returning to the actual Script view. It
connects the existing AI settings to a localhost HTTP/SSE provider fixture,
generates the next scene through the production client, edits the first scene,
preserves that edit through navigation, compares saved text without writing,
saves, and moves its original clip one second through the existing keyboard
shortcut. It verifies the segment placement moves while exact block text and
revision remain unchanged. It previews the impacted generated scene, requires
its canon to stay unchanged until the explicit Accept update click, then checks
only the target changes. The provider requires the exact current authored text
in both generation and preview prompts. It serves predefined text, not a model.

SQLite is opened read-only by the driver. All writes use public backend services
for project/notes setup or the real GUI/native command route for authoring.
Evidence records exact source identity, binary hash, native window PID/ID,
checkpoints, authored revision IDs, placement, canonical preview/accept checks,
and provider-fixture results. PNGs require visual inspection after execution.
Interrupted transport remains covered by existing frontend/provider/server
regressions; this driver does not inject a native lost acknowledgement.

A newer qualification push supersedes the obsolete run using standard Actions
concurrency cancellation on this isolated branch, avoiding duplicate native builds.
The push-only branch is `test/screenplay-authoring-native`; one bounded job runs
with read-only repository permissions and unpersisted checkout credentials.
The job has a 35-minute limit, two build workers, resource-floor checks, and a
six-minute GUI step with a 270-second driver deadline. Uploads are restricted to
three bounded PNGs, a sanitized 20-KiB launch-log tail, and evidence JSON, retained
for three days. No database, raw prompt, raw log, credentials or binary is uploaded.

Local preparation checks are formatting, Python syntax/helper behavior, source
identity guard, and compile-only example validation. These do not prove GUI or
native runtime execution. Exact local ORT acquisition already failed once with
`cdn.pyke.io` HTTP 403; it is not retried or bypassed. The existing hosted native
route succeeded for older source in run `37221991765`, providing route evidence
only. Real-model story quality remains unqualified by this fixture exercise.

Run `37250082342` built normally and passed all 382 real server library tests,
including WAL stale-writer/interleaving and interrupted descendant rollback.
The native app opened its empty project; traversal then hit a retired WebKit
accessibility object while the captured PID/window stayed alive and displayed
the workspace. This is partial GUI evidence, with no authored block or provider
call. The corrected driver retries only the observed retired-object error within
its existing deadline, refreshes from the root on the next poll, counts these
polls, and fails immediately if the native process disappears. Other errors stay
fatal. Complete GUI authoring and real-model quality remain unqualified.

Run `37250774496` again passes normal native compilation and all 382 actual
server tests. A repeated click on the still-loading home button stalls in
`xdotool mousemove --sync` with the same target coordinates; the 15-second
subprocess limit stops the driver before project opening. The underlying cause
is not established by the log.
The qualification now uses ordinary native mousemove and verifies exact X11
cursor coordinates before clicking, including the already-positioned case.
No application source or execution-security change is involved.

Run `37251636717` again passes normal native compilation and 382 server tests,
and now opens the project. Its exact accessible-text scene locator times out
while the inspected native image visibly shows the first scene. No authored text
or provider call occurs. The qualification uses canonical name containment in
native text and selects the smallest bounded individual control region, rejecting
whole-document/row matches and recording actual native text/role/geometry.
Canonical source identity/range checks after GUI save remain mandatory. Button
input waits for enabled state; the saved-text comparison uses supported native
scrolling to reveal its region. Application source is still unchanged.

Run `37252954727` again builds normally and passes all 382 server tests. Its
native evidence establishes that WebKit exposes the scene names together in one
1360-by-48-pixel row-wide Text object, with no individual clip object matching
the first scene. The locator correctly rejects that row's center and times out
before authoring. The qualification now asks the actual AT-SPI Text interface
for the exact unique scene label's substring bounds and clicks those verified
native glyph coordinates. Zero, row-wide, ambiguous and off-window ranges are
rejected. This supported geometry route changes only the qualification driver;
application source and execution security stay unchanged. Actual authoring flow
and real-model quality remain unqualified until execution succeeds.
