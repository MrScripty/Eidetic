use eidetic_bevy_bible_graph::{
    BibleGraphNativeWindowControlHandle, BibleGraphNativeWindowRunnerConfig,
    run_controlled_minimal_bible_graph_native_window, run_minimal_bible_graph_native_window,
};
use eidetic_core::contracts::{
    BibleGraphEdge, BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNode, BibleGraphNodeId,
    BibleGraphSchemaKey, BibleRenderGraphProjection,
};
use serde::Serialize;
use std::num::NonZeroU64;

fn main() {
    let command: Vec<String> = std::env::args().collect();
    let args = match NativeRendererSmokeArgs::parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(NativeRendererSmokeArgsError::HelpRequested) => {
            print_help();
            return;
        }
        Err(error) => {
            eprintln!("{error}");
            print_help();
            std::process::exit(2);
        }
    };

    if args.report_only {
        match serde_json::to_string_pretty(&NativeRendererSmokePreflightReport::from_args(
            args, command,
        )) {
            Ok(report) => {
                println!("{report}");
                return;
            }
            Err(error) => {
                eprintln!("failed to serialize native renderer smoke report: {error}");
                std::process::exit(1);
            }
        }
    }

    let mut config = BibleGraphNativeWindowRunnerConfig::minimal_smoke(args.run_on_any_thread);
    if let Some(auto_close_after_ms) = args.auto_close_after_ms {
        config = config.with_auto_close_after_ms(auto_close_after_ms);
    }

    if args.demo_graph {
        let control = BibleGraphNativeWindowControlHandle::new();
        control.set_projection(demo_projection());
        run_controlled_minimal_bible_graph_native_window(config, control);
    } else {
        run_minimal_bible_graph_native_window(config);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeRendererSmokeArgs {
    run_on_any_thread: bool,
    auto_close_after_ms: Option<NonZeroU64>,
    report_only: bool,
    demo_graph: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NativeRendererSmokePreflightReport {
    renderer_kind: &'static str,
    backend: &'static str,
    platform: &'static str,
    threading_model: &'static str,
    run_on_any_thread: bool,
    borderless_window: bool,
    width_px: u32,
    height_px: u32,
    auto_close_after_ms: Option<u64>,
    demo_graph: bool,
    command: Vec<String>,
    observed_result: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeRendererSmokeArgsError {
    HelpRequested,
    UnknownArgument(String),
    MissingAutoCloseDuration,
    InvalidAutoCloseDuration(String),
}

impl NativeRendererSmokeArgs {
    fn parse(
        arguments: impl IntoIterator<Item = String>,
    ) -> Result<Self, NativeRendererSmokeArgsError> {
        let mut parsed = Self {
            run_on_any_thread: true,
            auto_close_after_ms: None,
            report_only: false,
            demo_graph: false,
        };
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--main-thread" => parsed.run_on_any_thread = false,
                "--any-thread" => parsed.run_on_any_thread = true,
                "--report-only" => parsed.report_only = true,
                "--demo-graph" => parsed.demo_graph = true,
                "--auto-close-ms" => {
                    let Some(duration) = arguments.next() else {
                        return Err(NativeRendererSmokeArgsError::MissingAutoCloseDuration);
                    };
                    parsed.auto_close_after_ms = Some(parse_auto_close_duration(&duration)?);
                }
                "--help" | "-h" => return Err(NativeRendererSmokeArgsError::HelpRequested),
                unknown => {
                    let Some(duration) = unknown.strip_prefix("--auto-close-ms=") else {
                        return Err(NativeRendererSmokeArgsError::UnknownArgument(
                            unknown.to_string(),
                        ));
                    };
                    parsed.auto_close_after_ms = Some(parse_auto_close_duration(duration)?);
                }
            }
        }

        Ok(parsed)
    }
}

impl NativeRendererSmokePreflightReport {
    fn from_args(args: NativeRendererSmokeArgs, command: Vec<String>) -> Self {
        let config = BibleGraphNativeWindowRunnerConfig::minimal_smoke(args.run_on_any_thread);

        Self {
            renderer_kind: "bible_graph",
            backend: "bevy_winit",
            platform: current_platform(),
            threading_model: if args.run_on_any_thread {
                "worker_thread"
            } else {
                "main_thread"
            },
            run_on_any_thread: args.run_on_any_thread,
            borderless_window: config.borderless_window,
            width_px: config.width_px,
            height_px: config.height_px,
            auto_close_after_ms: args.auto_close_after_ms.map(NonZeroU64::get),
            demo_graph: args.demo_graph,
            command,
            observed_result: "not_run_report_only",
        }
    }
}

impl std::fmt::Display for NativeRendererSmokeArgsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HelpRequested => Ok(()),
            Self::UnknownArgument(argument) => write!(formatter, "unknown argument: {argument}"),
            Self::MissingAutoCloseDuration => {
                write!(
                    formatter,
                    "--auto-close-ms requires a nonzero millisecond value"
                )
            }
            Self::InvalidAutoCloseDuration(duration) => {
                write!(
                    formatter,
                    "invalid --auto-close-ms value: {duration}; expected a nonzero integer"
                )
            }
        }
    }
}

fn parse_auto_close_duration(duration: &str) -> Result<NonZeroU64, NativeRendererSmokeArgsError> {
    duration
        .parse::<NonZeroU64>()
        .map_err(|_| NativeRendererSmokeArgsError::InvalidAutoCloseDuration(duration.to_string()))
}

fn print_help() {
    println!(
        "Usage: eidetic-native-renderer-smoke [--any-thread|--main-thread] [--auto-close-ms <ms>] [--demo-graph] [--report-only]\n\
         Opens the minimal Eidetic Bevy bible graph smoke window and exits when the window closes.\n\
         --auto-close-ms exits the smoke window after a nonzero millisecond duration.\n\
         --demo-graph loads a fixed mixed-category graph projection before rendering.\n\
         --report-only prints the smoke preflight report without opening a window."
    );
}

fn demo_projection() -> BibleRenderGraphProjection {
    BibleRenderGraphProjection::from_graph(
        vec![
            demo_node("demo.character", None, "character", "Character", false, 0),
            demo_node("demo.location", None, "location", "Location", false, 1),
            demo_node("demo.prop", None, "prop", "Prop", false, 2),
            demo_node("demo.theme", None, "theme", "Theme", false, 3),
        ],
        vec![
            BibleGraphEdge {
                id: BibleGraphEdgeId::new("demo.edge.character.location").unwrap(),
                from_node_id: BibleGraphNodeId::new("demo.character").unwrap(),
                to_node_id: BibleGraphNodeId::new("demo.location").unwrap(),
                edge_kind: BibleGraphEdgeKind::LocatedIn,
                label: "located in".to_string(),
                directed: true,
                sort_order: 0,
            },
            BibleGraphEdge {
                id: BibleGraphEdgeId::new("demo.edge.character.prop").unwrap(),
                from_node_id: BibleGraphNodeId::new("demo.character").unwrap(),
                to_node_id: BibleGraphNodeId::new("demo.prop").unwrap(),
                edge_kind: BibleGraphEdgeKind::Owns,
                label: "owns".to_string(),
                directed: true,
                sort_order: 1,
            },
            BibleGraphEdge {
                id: BibleGraphEdgeId::new("demo.edge.prop.theme").unwrap(),
                from_node_id: BibleGraphNodeId::new("demo.prop").unwrap(),
                to_node_id: BibleGraphNodeId::new("demo.theme").unwrap(),
                edge_kind: BibleGraphEdgeKind::SupportsTheme,
                label: "supports".to_string(),
                directed: true,
                sort_order: 2,
            },
        ],
    )
}

fn demo_node(
    id: &str,
    parent_id: Option<&str>,
    schema_key: &str,
    name: &str,
    system_owned: bool,
    sort_order: u32,
) -> BibleGraphNode {
    BibleGraphNode {
        id: BibleGraphNodeId::new(id).unwrap(),
        parent_id: parent_id.map(|id| BibleGraphNodeId::new(id).unwrap()),
        schema_key: BibleGraphSchemaKey::new(schema_key).unwrap(),
        name: name.to_string(),
        system_owned,
        sort_order,
    }
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unsupported"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_to_worker_thread_smoke() {
        let args = NativeRendererSmokeArgs::parse([]).unwrap();

        assert!(args.run_on_any_thread);
        assert_eq!(args.auto_close_after_ms, None);
        assert!(!args.report_only);
        assert!(!args.demo_graph);
    }

    #[test]
    fn parse_accepts_main_thread_and_auto_close_duration() {
        let args = NativeRendererSmokeArgs::parse([
            "--main-thread".to_string(),
            "--auto-close-ms".to_string(),
            "250".to_string(),
        ])
        .unwrap();

        assert!(!args.run_on_any_thread);
        assert_eq!(args.auto_close_after_ms, NonZeroU64::new(250));
        assert!(!args.report_only);
    }

    #[test]
    fn parse_accepts_inline_auto_close_duration() {
        let args = NativeRendererSmokeArgs::parse(["--auto-close-ms=500".to_string()]).unwrap();

        assert_eq!(args.auto_close_after_ms, NonZeroU64::new(500));
    }

    #[test]
    fn parse_accepts_report_only_mode() {
        let args = NativeRendererSmokeArgs::parse(["--report-only".to_string()]).unwrap();

        assert!(args.report_only);
    }

    #[test]
    fn parse_accepts_demo_graph_mode() {
        let args = NativeRendererSmokeArgs::parse(["--demo-graph".to_string()]).unwrap();

        assert!(args.demo_graph);
    }

    #[test]
    fn parse_rejects_zero_auto_close_duration() {
        assert_eq!(
            NativeRendererSmokeArgs::parse(["--auto-close-ms".to_string(), "0".to_string()]),
            Err(NativeRendererSmokeArgsError::InvalidAutoCloseDuration(
                "0".to_string()
            ))
        );
    }

    #[test]
    fn preflight_report_records_smoke_runner_configuration() {
        let args = NativeRendererSmokeArgs::parse([
            "--main-thread".to_string(),
            "--auto-close-ms=250".to_string(),
            "--report-only".to_string(),
        ])
        .unwrap();

        let report = NativeRendererSmokePreflightReport::from_args(
            args,
            vec![
                "eidetic-native-renderer-smoke".to_string(),
                "--main-thread".to_string(),
            ],
        );

        assert_eq!(report.renderer_kind, "bible_graph");
        assert_eq!(report.backend, "bevy_winit");
        assert_eq!(report.platform, current_platform());
        assert_eq!(report.threading_model, "main_thread");
        assert!(!report.run_on_any_thread);
        assert!(!report.borderless_window);
        assert_eq!(report.width_px, 1280);
        assert_eq!(report.height_px, 720);
        assert_eq!(report.auto_close_after_ms, Some(250));
        assert!(!report.demo_graph);
        assert_eq!(report.observed_result, "not_run_report_only");
    }

    #[test]
    fn demo_projection_contains_mixed_backend_category_colors() {
        let visual =
            eidetic_bevy_bible_graph::build_bible_graph_visual_3d_snapshot(&demo_projection());

        let colors = visual
            .nodes
            .iter()
            .map(|node| node.fill_color)
            .collect::<std::collections::BTreeSet<_>>();

        assert!(colors.contains("#6495ed"));
        assert!(colors.contains("#22c55e"));
        assert!(colors.contains("#f97316"));
        assert!(colors.contains("#a855f7"));
    }
}
