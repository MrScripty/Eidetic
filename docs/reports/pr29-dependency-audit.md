# PR29 dependency audit disposition

## Identity and scope

Reviewed on 2026-10-14 against PR29 head `1148fb0e44efc25420e1b71494eef6d7cae99840`.
Baseline `ui/package-lock.json` blob: `de5b74ee43de2d7f0c3b8fe46ce67eabe7e1f73f`;
SHA256: `71205506e0eae5f68915ce93faa124a6a46a911ab35a0ae1a0ad1afd6837700b`.
Repaired lockfile SHA256:
`99dfbea2e4f52337273b69856f59a4a5c1a0d8b99bb461e94ced87116927a657`.

The baseline npm snapshot reported 14 high and 3 critical package rows, backed by
19 unique high/critical advisories. Inherited parent severity is distinguished
from an advisory against that parent's own code. The table accounts for all 17
rows; package versions and parent paths come from the exact lock and `npm explain`.
Upstream GitHub advisories and published npm version/peer/engine metadata were
checked directly. `fixAvailable: false` in the initial npm result did not mean
that upstream lacked a compatible fixed release; the initial readiness report's
inference to that effect is superseded here.

The write set is the UI manifest/lock, CI's frontend audit/static-build steps,
dependency governance and these readiness reports. No Rust/Pumas dependency or
application-source changes; credentials, model access and the trained-model gate
are unchanged.

## Deployment and input boundaries

`ui/svelte.config.js` uses `adapter-static`, writing only `dist/ui` for Tauri's
`frontendDist` in `src-tauri/tauri.conf.json`. Production runs no Node/SvelteKit
HTTP server. There are no application server loads, remote functions, cookie
setters or CSS/YAML/glob processing endpoints. Project/reference/model data is
handled by the existing Tauri/Rust commands, not those build-tool parsers.

This does not make every `devDependencies` entry non-runtime: SvelteKit's client
source imports devalue and the framework can serialize navigation/server data.
Devalue is classified as runtime-capable and patched. No current hostile
application path into its vulnerable Buffer/async/uneval operations was found;
this is not a general claim that devalue is absent from every emitted bundle.

Vite/SvelteKit also run in development. Kit 2.61.1's normal dev GET path calls
`server.respond` -> `is_action_json_request` -> `negotiate(Accept)` before checking
the method, making the Accept-header advisory reachable without remote forms.
The same-major Kit update below therefore fixes an actual available path.

## All baseline high/critical rows

| npm row / baseline version | Severity | Consumer and attack prerequisites | Repair |
| --- | --- | --- | --- |
| `@eslint/config-array 0.21.2` | High, inherited | Lint configuration glob patterns -> minimatch -> brace expansion; no project-text input. | Parent unchanged; patched brace dependency. |
| `@eslint/eslintrc 3.3.5` | High, inherited | Legacy lint YAML config and glob parsing, requiring malicious tool configuration. | Parent unchanged; YAML/brace patches. |
| `@tailwindcss/node 4.1.18` | High, inherited | CSS build source-map offsets through source-map-js; not runtime text processing. | Parent unchanged; source-map-js 1.2.2. |
| `@tailwindcss/vite 4.1.18` | High, inherited | Tailwind build dependency plus Vite dev-server paths. | Parent unchanged; source-map-js and Vite patches. |
| `@vitest/coverage-v8 3.2.4` | Critical, inherited | Test/coverage host via Vitest and magicast. | Coordinated 4.1.11 family; no Tinypool. |
| `brace-expansion 1.1.15 / 2.1.1 / 5.0.6` | High | ESLint/minimatch, TypeScript lint parser and coverage glob/test-exclude; attacker-controlled expansion patterns can exhaust CPU/stack/memory. | 1.1.21 and 5.0.12; obsolete coverage 2.x/5.x copies removed. |
| `devalue 5.8.1` | High | Runtime-capable framework dependency. Buffer serialization can expose backing storage; special uneval/async inputs can exhaust resources or reject unhandled promises. No current SSR Buffer/remote-input route, but retain no exemption. | 5.9.4. |
| `eslint 9.39.4` | High, inherited | Lint host through config-array/eslintrc/minimatch/YAML; development/CI, not shipped code. | Parent unchanged; patched affected dependencies, including humanfs 0.16.8. |
| `js-yaml 4.1.1` | High | ESLint legacy config parsing of attacker-controlled merge/omap YAML. | 4.3.2. |
| `magicast 0.3.5` | High, inherited | Coverage/config AST/source maps, not application reference documents. | Coverage-owned 0.5.5, with patched source-map-js. |
| `minimatch 3.1.5` | High, inherited | Lint glob selection, via brace expansion. | Parent unchanged; brace-expansion 1.1.21. |
| `nanoid 3.3.12` | High | PostCSS's generated input label calls `nanoid/non-secure` with constant positive size 6; app input cannot choose the negative/zero sizes in the advisories. | Compatible 3.3.20 despite absent current hostile-size path. |
| `postcss 8.5.15` | High | Node build/lint CSS and previous source-map loading; disclosure requires attacker CSS/map paths plus an observable generated result. No runtime CSS compilation endpoint. | 8.5.30 and patched nanoid/source-map-js; selector-parser 7.1.6 closes the related lower-severity lint chain. |
| `source-map-js 1.2.1` | High | Tailwind/PostCSS/magicast processing of malicious indexed source-map section offsets in the tool host. | 1.2.2. |
| `tinypool 1.1.1` | Critical | Vitest workers; an existing `Object.prototype` pollution primitive can materialize inherited worker/run options and obtain test-host execution. Not a desktop runtime worker. | Removed through supported Vitest 4 worker implementation; no forced 2.x override. |
| `vite 6.4.2` | High | Explicitly exposed dev server on Windows/NTFS with sensitive files in allowed directories; alternate paths bypass denial. Default launcher is loopback, but patch the supported Windows tool. | Same-line 6.4.4. |
| `vitest 3.2.4` | Critical | UI/browser/API server exposed or Windows UI mode; repository uses `vitest run` and no UI/browser/API configuration. Its worker dependency still needs repair independently. | 4.1.11 with matching coverage plugin. |

## Upstream advisory verification

These are the direct high/critical advisories in the baseline. Ranges/fixed
versions below describe the consumed major lines; selected versions exceed all
listed bounds. Linked upstream pages own complete multi-major ranges.

| Advisory | Affected consumed line / first fixed release |
| --- | --- |
| [GHSA-3jxr-9vmj-r5cp](https://github.com/advisories/GHSA-3jxr-9vmj-r5cp) | Brace: <1.1.16 / <2.1.2 / 3.x–<5.0.7; fixes 1.1.16, 2.1.2, 5.0.7. |
| [GHSA-mh99-v99m-4gvg](https://github.com/advisories/GHSA-mh99-v99m-4gvg) | Brace: <1.1.17 / 2.x<2.1.3 / 4.x–<5.0.8; fixes 1.1.17, 2.1.3, 5.0.8. |
| [GHSA-rgw5-rvv9-x895](https://github.com/advisories/GHSA-rgw5-rvv9-x895) | Brace: <1.1.18 / 2.x<2.1.4 / 4.x–<5.0.9; fixes 1.1.18, 2.1.4, 5.0.9. |
| [GHSA-6j4f-fj2g-mc7p](https://github.com/advisories/GHSA-6j4f-fj2g-mc7p) | Brace: <1.1.19 / 2.x<2.1.5 / 4.x–<5.0.10; fixes 1.1.19, 2.1.5, 5.0.10. |
| [GHSA-qhr7-859c-m2p7](https://github.com/advisories/GHSA-qhr7-859c-m2p7) | Brace: <1.1.20 / 2.x<2.1.6 / 4.x–<5.0.11; fixes 1.1.20, 2.1.6, 5.0.11. |
| [GHSA-j22f-vq7h-c4qm](https://github.com/advisories/GHSA-j22f-vq7h-c4qm) | Devalue >=5.1.0–5.9.2; fixed 5.9.3. |
| [GHSA-mcm9-63f2-9j32](https://github.com/advisories/GHSA-mcm9-63f2-9j32) | Devalue <=5.9.2; fixed 5.9.3. |
| [GHSA-x5rw-q4pp-hg5g](https://github.com/advisories/GHSA-x5rw-q4pp-hg5g) | Devalue >=5.8.0–5.9.2; fixed 5.9.3. |
| [GHSA-52cp-r559-cp3m](https://github.com/advisories/GHSA-52cp-r559-cp3m) | YAML 4.x<4.3.0; fixed 4.3.0. |
| [GHSA-5p4m-2wfm-xmqj](https://github.com/advisories/GHSA-5p4m-2wfm-xmqj) | YAML 4.x<4.3.1; fixed 4.3.1. |
| [GHSA-2883-xcg3-v3hh](https://github.com/advisories/GHSA-2883-xcg3-v3hh) | YAML 4.x<4.3.2; fixed 4.3.2. |
| [GHSA-28wg-ghj8-5hjv](https://github.com/advisories/GHSA-28wg-ghj8-5hjv) | Nanoid <3.3.16; fixed 3.3.16. |
| [GHSA-2v37-7h3g-55p8](https://github.com/advisories/GHSA-2v37-7h3g-55p8) | Nanoid <3.3.18; fixed 3.3.18. |
| [GHSA-r28c-9q8g-f849](https://github.com/advisories/GHSA-r28c-9q8g-f849) | PostCSS <=8.5.17; fixed 8.5.18. Selected 8.5.30 also fixes the later moderate map-loading advisory. |
| [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q) | source-map-js 1.x<1.2.2; fixed 1.2.2. |
| [GHSA-5gmw-xhrv-c9v3](https://github.com/advisories/GHSA-5gmw-xhrv-c9v3) | Tinypool <=2.1.0; first fixed 2.1.1, insufficient for the next advisory. No patched 1.x. |
| [GHSA-85c8-ppgw-ccpr](https://github.com/advisories/GHSA-85c8-ppgw-ccpr) | Tinypool <2.1.2; fixed 2.1.2. No patched 1.x; selected graph removes it. |
| [GHSA-fx2h-pf6j-xcff](https://github.com/advisories/GHSA-fx2h-pf6j-xcff) | Vite <=6.4.2; fixed 6.4.3. |
| [GHSA-5xrq-8626-4rwp](https://github.com/advisories/GHSA-5xrq-8626-4rwp) | Current reviewed feed: Vitest <3.2.6 or 4.x<4.1.0; selected 4.1.11. |

## Version decision and compatibility evidence

Vitest 3.2.7 still requires Tinypool `^1.1.1`; patching only Vitest 3 cannot
remove that critical finding. A forced Tinypool major would contradict the
parent's supported range. [Vitest 4's migration guide](https://v4.vitest.dev/guide/migration)
documents removal of Tinypool, Vite >=6 and Node >=20. The repo already pins
Node 24.12.0 and Vite 6, and has no custom pools, browser/UI server, removed
workspace/coverage options or custom runner APIs. Vitest and coverage-v8 are
updated together to 4.1.11. Existing assertions and discovery remain unchanged.
The current 4.1.11 cohort also fixes the moderate mocker path issue in 3.x.

Other changes stay in their existing families: Vite 6.4.4, Kit 2.70.3, devalue
5.9.4, PostCSS 8.5.30, nanoid 3.3.20, source-map-js 1.2.2, YAML 4.3.2, brace
1.1.21/5.0.12, humanfs/node 0.16.8 and selector-parser 7.1.6. The latter two
are compatible fixes within the already-reviewed ESLint/PostCSS chains.
Supporting Babel/Chai/magicast/worker-tool changes belong to the selected Vitest
cohort. Removed coverage/glob packages and Tinypool account for most lockfile
deletions. No direct Svelte, ESLint, Tailwind, TypeScript or Rust family upgrade,
no alternative registry, no overrides, and no `audit fix --force` are used.

The additional Kit repair closes [Accept-header negotiation](https://github.com/advisories/GHSA-29g2-3rmr-qm68)
(fixed 2.70.2) and its remote-form issues; supported peers still include this
repo's Svelte 5, plugin 5, Vite 6 and Node 24. The same-family supporting repairs
close [humanfs symlink copies](https://github.com/advisories/GHSA-p498-v437-472g)
and [selector-parser CPU exhaustion](https://github.com/advisories/GHSA-rj75-hqrm-r3gf),
without relying on their absence from application input paths as an exception.

## Remaining finding and required decision

The final exact-lock audit reports zero critical, high or moderate findings and
three low package rows: cookie, Kit and adapter-static. There is no remaining
high/critical exception. The remaining direct advisory is
[cookie GHSA-pxg6-pf52-xh8x](https://github.com/advisories/GHSA-pxg6-pf52-xh8x),
low severity, `cookie <0.7.0`; the lock contains 0.6.0 through Kit's `^0.6.0`.
Npm may also report Kit/adapter parent rows for that same advisory.

The prerequisite is an application passing untrusted cookie **names, paths or
domains** to the Node cookie serializer. No such caller, HTTP production server
or server load/form endpoint exists in this static Tauri application. The build
server output under `.svelte-kit` is not the shipped `dist/ui` payload. Normal
development does not introduce application cookie setters either.

This is a specific existing low-severity disposition, not blanket acceptance.
The frontend dependency owner must reassess it before introducing SSR/HTTP
deployment, remote forms or application-supplied cookie metadata, and during
the next Kit dependency update. As of this review, the latest Kit 2 release is
2.70.3 and retains cookie 0.6; Kit 3.0.1 uses cookie `^2.0.1`. Parent decision:
schedule a separately reviewed Kit 3 migration or obtain a supported Kit 2
backport before claiming that this dependency advisory is closed. Forcing
cookie >=0.7 under Kit 2 is not selected without a supported compatibility
contract. The existing high/critical gate is unchanged; no threshold is lowered.

## Verification and retained gates

The selected graph is installed with `npm ci` under pinned Node 24.12.0. Full
lint/format/typecheck, all 107 configured files / 608 tests and the static build
pass after the final dependency changes. The final audit has three low inherited/
direct rows for one cookie advisory, and zero moderate/high/critical findings;
`npm audit --audit-level=high` exits successfully. `npm ls` and the existing
`scripts/check-dependencies.sh` pass, with explicit optional Rust-tool warnings.
No test assertions, discovery filters,
snapshots or application behavior were weakened for the runner migration.
The matched coverage provider also passed a five-test Pumas lifecycle/prompt
smoke with V8 coverage enabled before the final lint-only transitive patches.

A bounded public-API regression used a synthetic two-byte Node Buffer view over
a 64-byte backing store, with no real process data. Devalue 5.8.1's
`parse(stringify(buffer))` reproduced a 64-byte backing-store disclosure;
5.9.4 preserves only the selected two bytes. Its `uneval` round trip also
preserves exactly those two bytes. These are dependency tests, not model output.

Reproduce the source verification from `ui`:

```bash
npm ci
npm audit --audit-level=high
npm ls
npm run lint
npm run format:check
npm run typecheck
npm run test
npm run build
```

CI now enforces the existing npm high/critical gate and the actual static build.
The PR record owns the exact pushed head, final independent review and hosted
run. Local full-native testing remains blocked by missing glib/GTK/WebKit;
the existing hosted Linux/Windows Rust checks are retained. Optional local Rust
audit/deny/machete tools remain absent; npm findings are not a claim that those
independent Rust audits were executed.

PR29 remains draft for parent review. Trained chat/embedding inference remains
unexecuted, with the original access denial and operator acceptance procedure
preserved in [the readiness report](pr29-merge-readiness.md). No model access
bypass, release or PR merge is performed.
