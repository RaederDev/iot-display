use super::*;

#[test]
fn departure_columns_and_pi_hole_stats_match_the_reference_layout() {
    let temp = tempfile::tempdir().unwrap();
    let context = egui::Context::default();
    let creation = eframe::CreationContext::_new_kittest(context.clone());
    let mut app = App::new(
        &creation,
        Config::default(),
        Startup {
            config_path: temp.path().join("config.json"),
            cache_path: temp.path().join("cache.json"),
            debug_flag: false,
            windowed: true,
            first_run: false,
            load_error: None,
        },
    )
    .unwrap();
    app.config.departures.enabled = true;
    app.config.pi_hole.enabled = true;
    app.config.departures.stops.push(crate::config::StopConfig {
        id: 60200267,
        kind: StopKind::Diva,
        label: "Eichenstraße".into(),
        lines: vec![],
    });
    let now = chrono::Utc::now();
    let destination = "A VERY LONG DESTINATION THAT MUST NOT COVER THE COUNTDOWN ".repeat(4);
    app.data.entries.insert(
        crate::integrations::key(Provider::Departures, &app.config),
        Entry {
            fetched_at: now.timestamp(),
            data: Data::Departures(vec![crate::integrations::departures::Board {
                id: 399,
                diva: "60200267".into(),
                title: "Eichenstraße".into(),
                available_lines: vec![],
                departures: [
                    ("14A", destination.as_str(), 620),
                    ("WLB", "WIEN OPER", 140),
                ]
                .into_iter()
                .map(
                    |(line, destination, seconds)| crate::integrations::departures::Departure {
                        line: line.into(),
                        destination: destination.into(),
                        direction: "1".into(),
                        at: now + chrono::Duration::seconds(seconds),
                        planned: false,
                    },
                )
                .collect(),
            }]),
        },
    );
    app.data.entries.insert(
        crate::integrations::key(Provider::PiHole, &app.config),
        Entry {
            fetched_at: now.timestamp()
                - crate::integrations::Request::configured(Provider::PiHole, &app.config)
                    .refresh_seconds() as i64
                - 1,
            data: Data::PiHole(crate::integrations::pi_hole::Stats {
                queries: 7319,
                blocked: 2330,
                unique_domains: 822,
            }),
        },
    );
    for size in [
        egui::vec2(800.0, 600.0),
        egui::vec2(1024.0, 600.0),
        egui::vec2(1920.0, 1080.0),
    ] {
        frame(&mut app, &context, size, vec![]);
        let output = frame(&mut app, &context, size, vec![]);
        assert!(!output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("refreshing"))
        }));
        let text = |label: &str| {
            output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == label => Some(text),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("Missing label {label}"))
        };
        let bounds = |label: &str| {
            let text = text(label);
            text.galley.rect.translate(text.pos.to_vec2())
        };
        let line = bounds("14A");
        let time = bounds("10");
        assert!((time.right() - bounds("2").right()).abs() < 1.0);
        assert!(
            (bounds("EICHENSTRASSE").center().x - (line.left() + time.right()) / 2.0).abs() < 1.0,
            "heading={:?}, line={line:?}, time={time:?}",
            bounds("EICHENSTRASSE")
        );
        assert!(bounds("EICHENSTRASSE").bottom() < line.top());
        assert!(text(&destination).galley.elided);
        assert_eq!(
            text(&destination).galley.rows[0].glyphs.last().unwrap().chr,
            '…'
        );
        assert!(bounds(&destination).right() < time.left());
        assert_eq!(text("14A").galley.job.sections[0].format.font_id.size, 14.0);
        let stats_left = bounds("7319").left();
        assert!((stats_left - bounds("2330").left()).abs() < 1.0);
        assert!((stats_left - bounds("822").left()).abs() < 1.0);
        assert!(bounds("Blocked queries").right() < stats_left);
        assert_eq!(
            text("7319").galley.job.sections[0].format.font_id.size,
            18.0
        );
    }
    let size = egui::vec2(1024.0, 600.0);
    click(&mut app, &context, "14A", size);
    assert_eq!(
        app.widgets
            .view::<crate::widgets::widget_departures::View>(crate::config::Widget::Departures)
            .details[&(1, 1, 0)]
            .line,
        "14A"
    );
    click(&mut app, &context, "Back to all departures", size);
    assert!(
        app.widgets
            .view::<crate::widgets::widget_departures::View>(crate::config::Widget::Departures)
            .details
            .is_empty()
    );
    click(&mut app, &context, "10", size);
    assert_eq!(
        app.widgets
            .view::<crate::widgets::widget_departures::View>(crate::config::Widget::Departures)
            .details[&(1, 1, 0)]
            .line,
        "14A"
    );
    app.data.errors.insert(
        crate::integrations::key(Provider::PiHole, &app.config),
        crate::ui::data::DataError {
            provider: Provider::PiHole,
            message: "Pi-hole connection failed".into(),
            retry_at: now.timestamp() + 30,
        },
    );
    let output = frame(&mut app, &context, size, vec![]);
    assert!(output.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("Pi-hole connection failed"))
    }));
}

#[test]
fn catalogue_selection_and_line_text_filter_keep_attribution_in_settings() {
    let temp = tempfile::tempdir().unwrap();
    let context = egui::Context::default();
    let creation = eframe::CreationContext::_new_kittest(context.clone());
    let mut app = App::new(
        &creation,
        Config::default(),
        Startup {
            config_path: temp.path().join("config.json"),
            cache_path: temp.path().join("cache.json"),
            debug_flag: false,
            windowed: true,
            first_run: true,
            load_error: None,
        },
    )
    .unwrap();
    app.settings.tab = Some(Tab::Widget(crate::config::Widget::Departures));
    app.widgets
        .view::<crate::widgets::widget_departures::View>(crate::config::Widget::Departures)
        .editor
        .query = "eichen".into();
    app.data.entries.insert(
        crate::integrations::key(Provider::Catalogue, &app.config),
        Entry {
            fetched_at: chrono::Utc::now().timestamp(),
            data: Data::Catalogue(
                crate::integrations::departures::parse_catalogue(include_bytes!(
                    "../../../../tests/fixtures/wl-stations.csv"
                ))
                .unwrap(),
            ),
        },
    );
    let size = egui::vec2(1920.0, 1080.0);
    click(&mut app, &context, "Add Eichenstraße · DIVA 60200267", size);
    assert_eq!(app.settings.draft.departures.stops[0].id, 60200267);
    assert_eq!(app.settings.draft.departures.stops[0].kind, StopKind::Diva);
    click(&mut app, &context, "12a, 14a", size);
    frame(
        &mut app,
        &context,
        size,
        vec![egui::Event::Text("12a".into())],
    );
    assert_eq!(app.settings.draft.departures.stops[0].lines, vec!["12a"]);
    let output = frame(&mut app, &context, size, vec![]);
    label_position(&output, "Datenquelle: Stadt Wien");
    for shape in &output.shapes {
        if let egui::Shape::Text(text) = &shape.shape {
            assert!(!text.galley.job.text.contains("Load stop catalogue"));
            assert!(!text.galley.job.text.contains("Reload catalogue"));
            assert!(!text.galley.job.text.contains("Five minutes matches"));
        }
    }
    app.config = app.settings.draft.clone();
    app.config.weather.enabled = true;
    app.settings.tab = None;
    let output = frame(&mut app, &context, size, vec![]);
    for shape in &output.shapes {
        if let egui::Shape::Text(text) = &shape.shape {
            assert!(!text.galley.job.text.contains("Open-Meteo"));
            assert!(!text.galley.job.text.contains("Datenquelle"));
            assert!(!text.galley.job.text.contains("non-commercial"));
        }
    }
}
