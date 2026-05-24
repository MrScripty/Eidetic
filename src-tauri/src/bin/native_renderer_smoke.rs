use eidetic_bevy_bible_graph::{
    BibleGraphNativeWindowRunnerConfig, run_minimal_bible_graph_native_window,
};
use std::num::NonZeroU64;

fn main() {
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

    let mut config = BibleGraphNativeWindowRunnerConfig::minimal_smoke(args.run_on_any_thread);
    if let Some(auto_close_after_ms) = args.auto_close_after_ms {
        config = config.with_auto_close_after_ms(auto_close_after_ms);
    }

    run_minimal_bible_graph_native_window(config);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeRendererSmokeArgs {
    run_on_any_thread: bool,
    auto_close_after_ms: Option<NonZeroU64>,
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
        };
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--main-thread" => parsed.run_on_any_thread = false,
                "--any-thread" => parsed.run_on_any_thread = true,
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
        "Usage: eidetic-native-renderer-smoke [--any-thread|--main-thread] [--auto-close-ms <ms>]\n\
         Opens the minimal Eidetic Bevy bible graph smoke window and exits when the window closes.\n\
         --auto-close-ms exits the smoke window after a nonzero millisecond duration."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_to_worker_thread_smoke() {
        let args = NativeRendererSmokeArgs::parse([]).unwrap();

        assert!(args.run_on_any_thread);
        assert_eq!(args.auto_close_after_ms, None);
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
    }

    #[test]
    fn parse_accepts_inline_auto_close_duration() {
        let args = NativeRendererSmokeArgs::parse(["--auto-close-ms=500".to_string()]).unwrap();

        assert_eq!(args.auto_close_after_ms, NonZeroU64::new(500));
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
}
