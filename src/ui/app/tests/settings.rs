use super::*;

#[test]
fn settings_save_cancel_preview_and_keyboard_access_at_display_sizes() {
    let temp = tempfile::tempdir().unwrap();
    let context = egui::Context::default();
    let creation = eframe::CreationContext::_new_kittest(context.clone());
    let path = temp.path().join("config/config.json");
    let mut app = App::new(
        &creation,
        Config::default(),
        Startup {
            config_path: path.clone(),
            cache_path: temp.path().join("cache/cache.json"),
            debug_flag: false,
            windowed: true,
            first_run: true,
            load_error: None,
        },
    )
    .unwrap();
    let small = egui::vec2(1024.0, 600.0);
    frame(&mut app, &context, small, vec![]);
    app.settings.draft.username = "Unsaved".into();
    app.settings.draft.screen_wake = true;
    click(&mut app, &context, "Cancel", small);
    assert!(app.settings.tab.is_none());
    assert_eq!(app.settings.draft.username, "");
    assert!(!app.settings.draft.screen_wake);
    assert!(!path.exists());
    frame(
        &mut app,
        &context,
        small,
        vec![egui::Event::Key {
            key: egui::Key::Comma,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        }],
    );
    assert!(app.settings.tab.is_some(), "Ctrl+, should open settings");
    app.settings.draft.username = "Saved".into();
    app.settings.draft.screen_wake = true;
    click(&mut app, &context, "Save", small);
    assert_eq!(Config::load(&path).unwrap().username, "Saved");
    assert!(Config::load(&path).unwrap().screen_wake);
    assert!(app.settings.tab.is_none());
    assert!(app.notice.is_empty());
    app.open_settings();
    for size in [small, egui::vec2(1920.0, 1080.0)] {
        for tab in Tab::all() {
            app.settings.tab = Some(tab);
            frame(&mut app, &context, size, vec![]);
        }
    }
    app.settings.tab = Some(Tab::Layout);
    click(&mut app, &context, "Add row", small);
    assert_eq!(app.settings.draft.layout.len(), app.config.layout.len() + 1);
    click(&mut app, &context, "Cancel", small);
    assert_eq!(app.settings.draft.layout.len(), app.config.layout.len());
    app.open_settings();
    app.settings.draft.dark = true;
    frame(&mut app, &context, small, vec![]);
    assert!(context.global_style().visuals.dark_mode);
    app.settings.tab = None;
    app.config.status_bar = false;
    let output = frame(&mut app, &context, small, vec![]);
    label_position(&output, "Settings");
    label_position(&output, "Quit");
}

#[test]
fn invalid_configuration_is_preserved_until_explicit_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("config.json");
    std::fs::write(&path, "invalid").unwrap();
    let context = egui::Context::default();
    let creation = eframe::CreationContext::_new_kittest(context.clone());
    let mut app = App::new(
        &creation,
        Config::default(),
        Startup {
            config_path: path.clone(),
            cache_path: temp.path().join("cache.json"),
            debug_flag: false,
            windowed: true,
            first_run: true,
            load_error: Some("Invalid configuration".into()),
        },
    )
    .unwrap();
    let size = egui::vec2(1024.0, 600.0);
    frame(&mut app, &context, size, vec![]);
    click(&mut app, &context, "Save", size);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "invalid");
    app.settings.replace_invalid = true;
    click(&mut app, &context, "Save", size);
    assert!(Config::load(&path).is_ok());
}
