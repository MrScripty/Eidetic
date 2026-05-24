use eidetic_bevy_bible_graph::{
    BibleGraphNativeWindowRunnerConfig, run_minimal_bible_graph_native_window,
};

fn main() {
    let mut run_on_any_thread = true;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--main-thread" => run_on_any_thread = false,
            "--any-thread" => run_on_any_thread = true,
            "--help" | "-h" => {
                print_help();
                return;
            }
            unknown => {
                eprintln!("unknown argument: {unknown}");
                print_help();
                std::process::exit(2);
            }
        }
    }

    run_minimal_bible_graph_native_window(BibleGraphNativeWindowRunnerConfig::minimal_smoke(
        run_on_any_thread,
    ));
}

fn print_help() {
    println!(
        "Usage: eidetic-native-renderer-smoke [--any-thread|--main-thread]\n\
         Opens the minimal Eidetic Bevy bible graph smoke window and exits when the window closes."
    );
}
