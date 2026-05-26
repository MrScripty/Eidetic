fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "eidetic_desktop=info,eidetic_server=info".into()),
        )
        .init();

    match std::env::args().nth(1).as_deref() {
        Some("--smoke") => {
            match eidetic_desktop::smoke_report_json() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("failed to serialize smoke report: {error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--graph-renderer-smoke") => {
            match eidetic_desktop::graph_renderer_lifecycle_smoke_report_json() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--timeline-renderer-smoke") => {
            match eidetic_desktop::timeline_renderer_lifecycle_smoke_report_json() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--help") => {
            println!(
                "Usage: eidetic-desktop [--smoke|--graph-renderer-smoke|--timeline-renderer-smoke]"
            );
            return;
        }
        _ => {}
    }

    eidetic_desktop::run();
}
