use super::*;

#[test]
fn weather_location_selection_clears_search_and_supports_clear_save_cancel() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("config.json");
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
            load_error: None,
        },
    )
    .unwrap();
    let location = crate::integrations::weather::Location {
        name: "Graz".into(),
        latitude: 47.0707,
        longitude: 15.4395,
        country: "Austria".into(),
        admin1: "Styria".into(),
    };
    for size in [egui::vec2(1024.0, 600.0), egui::vec2(1920.0, 1080.0)] {
        app.settings.tab = Some(Tab::Widget(crate::config::Widget::Weather));
        app.widgets
            .view::<crate::widgets::widget_weather::View>(crate::config::Widget::Weather)
            .editor
            .query = "graz".into();
        assert!(
            !app.widgets
                .view::<crate::widgets::widget_weather::View>(crate::config::Widget::Weather)
                .editor
                .accept_results("geocoding:vienna", std::slice::from_ref(&location))
        );
        assert!(
            app.widgets
                .view::<crate::widgets::widget_weather::View>(crate::config::Widget::Weather)
                .editor
                .accept_results("geocoding:graz", std::slice::from_ref(&location))
        );
        click(&mut app, &context, &location.label(), size);
        assert!(
            app.widgets
                .view::<crate::widgets::widget_weather::View>(crate::config::Widget::Weather)
                .editor
                .query
                .is_empty()
        );
        assert!(
            app.widgets
                .view::<crate::widgets::widget_weather::View>(crate::config::Widget::Weather)
                .editor
                .results
                .is_empty()
        );
        assert!(
            !app.widgets
                .view::<crate::widgets::widget_weather::View>(crate::config::Widget::Weather)
                .editor
                .accept_results("geocoding:graz", std::slice::from_ref(&location))
        );
        assert!(app.settings.draft.weather.enabled);
        assert_eq!(app.settings.draft.weather.label, "Graz");
        assert_eq!(app.settings.draft.weather.latitude, location.latitude);
        assert_eq!(app.settings.draft.weather.longitude, location.longitude);
        let output = frame(&mut app, &context, size, vec![]);
        label_position(&output, "Selected location: Graz");
        label_position(&output, "Latitude: 47.07070 · Longitude: 15.43950");
        app.config.weather = app.settings.draft.weather.clone();
        app.config.weather.enabled = false;
        app.settings.draft.weather.latitude = 47.1;
        app.settings.draft.weather.longitude = 15.4;
        let output = frame(&mut app, &context, size, vec![]);
        label_position(&output, "Latitude: 47.10000 · Longitude: 15.40000");
        app.settings.draft.weather.fahrenheit = true;
        click(&mut app, &context, "Clear location", size);
        assert!(!app.settings.draft.weather.enabled);
        assert!(app.settings.draft.weather.label.is_empty());
        assert_eq!(app.settings.draft.weather.latitude, 0.0);
        assert_eq!(app.settings.draft.weather.longitude, 0.0);
        assert!(app.settings.draft.weather.fahrenheit);
        let output = frame(&mut app, &context, size, vec![]);
        label_position(&output, "No location selected.");
        click(&mut app, &context, "Cancel", size);
        assert_eq!(app.settings.draft.weather.label, "Graz");
        assert_eq!(app.settings.draft.weather.latitude, location.latitude);
        app.open_settings();
        app.settings.tab = Some(Tab::Widget(crate::config::Widget::Weather));
        click(&mut app, &context, "Clear location", size);
        click(&mut app, &context, "Save", size);
        let saved = Config::load(&path).unwrap();
        assert!(!saved.weather.enabled);
        assert!(saved.weather.label.is_empty());
        app.open_settings();
        app.settings.tab = Some(Tab::Widget(crate::config::Widget::Weather));
        let output = frame(&mut app, &context, size, vec![]);
        label_position(&output, "No location selected.");
    }
}
