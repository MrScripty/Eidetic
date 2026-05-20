fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "eidetic_desktop=info,eidetic_server=info".into()),
        )
        .init();

    eidetic_desktop::run();
}
