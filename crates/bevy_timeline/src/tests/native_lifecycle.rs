use crate::{
    TimelineNativeWindowControl, TimelineNativeWindowControlHandle,
    TimelineNativeWindowRunnerConfig, configure_controlled_minimal_timeline_native_window_app,
};

#[cfg(feature = "native_render")]
#[test]
fn native_window_runner_config_records_minimal_smoke_window_intent() {
    let config = TimelineNativeWindowRunnerConfig::minimal_smoke(true);

    assert_eq!(config.title, "Eidetic Timeline");
    assert_eq!(config.width_px, 1280);
    assert_eq!(config.height_px, 360);
    assert!(!config.borderless_window);
    assert!(config.run_on_any_thread);
    assert_eq!(config.auto_close_after_ms, None);
    assert_eq!(config.initial_projection, None);

    let auto_close_ms = std::num::NonZeroU64::new(250).unwrap();
    let config = config.with_auto_close_after_ms(auto_close_ms);

    assert_eq!(config.auto_close_after_ms, Some(auto_close_ms));
}

#[cfg(feature = "native_render")]
#[test]
fn native_window_control_handle_records_close_requests() {
    let control = TimelineNativeWindowControlHandle::new();

    assert!(!control.close_requested());
    assert!(!control.ready());
    assert!(!control.visible());

    control.request_close();
    control.mark_ready();
    control.mark_visible(true);

    assert!(control.close_requested());
    assert!(control.ready());
    assert!(control.visible());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_app_installs_close_control_resource() {
    let control = TimelineNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    configure_controlled_minimal_timeline_native_window_app(
        &mut app,
        TimelineNativeWindowRunnerConfig::minimal_smoke(true),
        control.clone(),
    );

    assert!(
        app.world()
            .contains_resource::<TimelineNativeWindowControl>()
    );
    assert!(!control.close_requested());
    assert!(!control.ready());

    control.request_close();

    assert!(control.close_requested());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_os_close_requests_shutdown() {
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    control.mark_visible(true);
    window_control.request_close_from_os_window();

    assert!(control.close_requested());
    assert!(!control.visible());
}
