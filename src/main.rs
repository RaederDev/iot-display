use iot_display::{config::Config, storage, ui::App};

fn main() {
    if let Err(error) = run() {
        eprintln!("iot-display: {error}");
        std::process::exit(1);
    }
}

fn run() -> iot_display::Result<()> {
    let mut debug = false;
    let mut windowed = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--debug" => debug = true,
            "--windowed" => windowed = true,
            "--help" | "-h" => {
                println!(
                    "iot-display [--debug] [--windowed]\n\n--debug     Print each outgoing HTTP request to stdout (credentials redacted)\n--windowed  Override fullscreen for this launch\n\nSettings and layouts are edited in the application."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {argument}. Use --help.")),
        }
    }
    let config_path = storage::app_directory("XDG_CONFIG_HOME", ".config")?.join("config.json");
    let cache_path = storage::app_directory("XDG_CACHE_HOME", ".cache")?.join("cache.json");
    let (config, load_error, first_run) = if config_path.exists() {
        match Config::load(&config_path) {
            Ok(config) => (config, None, false),
            Err(error) => (Config::default(), Some(error), true),
        }
    } else {
        (Config::default(), None, true)
    };
    let fullscreen = config.fullscreen && !windowed;
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("iot-display")
            .with_title("IOT Display")
            .with_inner_size([1024.0, 600.0])
            .with_fullscreen(fullscreen),
        ..Default::default()
    };
    eframe::run_native(
        "IOT Display",
        options,
        Box::new(move |context| {
            let app = App::new(
                context,
                config,
                iot_display::ui::Startup {
                    config_path,
                    cache_path,
                    debug_flag: debug,
                    windowed,
                    first_run,
                    load_error,
                },
            )?;
            Ok(Box::new(app))
        }),
    )
    .map_err(|error| error.to_string())
}
