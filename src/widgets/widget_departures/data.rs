use crate::{
    Result,
    config::{DeparturesConfig, StopConfig, StopKind},
    http::{Http, HttpError},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use wl_realtime_ogd::models::{
    BuildRequestUrl, MessageCode, MonitorRequest, MonitorResponse, Response,
};

pub const CATALOGUE_URL: &str =
    "https://www.wienerlinien.at/ogd_realtime/doku/ogd/wienerlinien-ogd-haltestellen.csv";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stop {
    #[serde(rename = "DIVA")]
    pub diva: u32,
    #[serde(rename = "PlatformText", alias = "StopText")]
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Departure {
    pub line: String,
    pub destination: String,
    pub direction: String,
    pub at: DateTime<Utc>,
    pub planned: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub id: u32,
    pub diva: String,
    pub title: String,
    pub departures: Vec<Departure>,
    pub available_lines: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub stop_id: u32,
    pub line: String,
    pub direction: String,
}

pub fn display_departures<'a>(
    board: &'a Board,
    lines: &[String],
    now: DateTime<Utc>,
    selection: Option<&Selection>,
) -> Vec<&'a Departure> {
    let mut shown = std::collections::HashSet::new();
    upcoming(board, lines, now)
        .into_iter()
        .filter(|departure| {
            if let Some(selection) = selection {
                departure.line == selection.line && departure.direction == selection.direction
            } else {
                shown.insert((&departure.line, &departure.direction))
            }
        })
        .collect()
}

fn monitor_url(request: MonitorRequest) -> reqwest::Url {
    reqwest::Url::parse(&format!(
        "https://www.wienerlinien.at/ogd_realtime{}",
        request.build_request_url()
    ))
    .unwrap()
}

pub fn monitor_urls(config: &DeparturesConfig) -> Vec<reqwest::Url> {
    let mut stop_ids = Vec::new();
    let mut divas = Vec::new();
    for stop in &config.stops {
        match stop.kind {
            StopKind::StopId => stop_ids.push(stop.id),
            StopKind::Diva => divas.push(stop.id),
        }
    }
    stop_ids.sort_unstable();
    stop_ids.dedup();
    divas.sort_unstable();
    divas.dedup();

    let mut urls = Vec::new();
    if !stop_ids.is_empty() {
        urls.push(monitor_url(MonitorRequest {
            stop_id: stop_ids,
            ..Default::default()
        }));
    }
    // The monitor API accepts one DIVA number per request.
    for diva in divas {
        urls.push(monitor_url(MonitorRequest {
            diva: Some(diva),
            ..Default::default()
        }));
    }
    urls
}

pub fn parse_monitor(bytes: &[u8]) -> Result<Vec<Board>> {
    let envelope: Response =
        serde_json::from_slice(bytes).map_err(|_| "Invalid departures response")?;
    if envelope.message.message_code != MessageCode::OK {
        return Err(format!(
            "Wiener Linien: {:?}",
            envelope.message.message_code
        ));
    }
    let response: MonitorResponse =
        serde_json::from_slice(bytes).map_err(|_| "Invalid departures data")?;
    let mut boards: Vec<Board> = vec![];
    for monitor in response.data.monitors {
        let properties = monitor.location_stop.properties;
        let id =
            u32::try_from(properties.attributes.rbl).map_err(|_| "Invalid stop ID in response")?;
        if id == 0 {
            return Err("Invalid stop ID in response".into());
        }
        let position = boards.iter().position(|board| board.id == id);
        let index = position.unwrap_or_else(|| {
            boards.push(Board {
                id,
                diva: properties.name,
                title: properties.title,
                departures: vec![],
                available_lines: vec![],
            });
            boards.len() - 1
        });
        for line in monitor.lines {
            boards[index].available_lines.push(line.name.clone());
            for departure in line.departures.departure {
                let time = departure.departure_time;
                // Short workings can have a different destination from the line defaults.
                let vehicle = departure.vehicle;
                boards[index].departures.push(Departure {
                    line: vehicle
                        .as_ref()
                        .map_or_else(|| line.name.clone(), |vehicle| vehicle.name.clone()),
                    destination: vehicle
                        .as_ref()
                        .map_or_else(|| line.towards.clone(), |vehicle| vehicle.towards.clone()),
                    direction: vehicle.as_ref().map_or_else(
                        || line.richtungs_id.clone(),
                        |vehicle| vehicle.richtungs_id.clone(),
                    ),
                    at: time
                        .time_real
                        .unwrap_or(time.time_planned)
                        .with_timezone(&Utc),
                    planned: time.time_real.is_none(),
                });
            }
        }
    }
    for board in &mut boards {
        board.departures.sort_by_key(|departure| departure.at);
        board.available_lines.sort();
        board.available_lines.dedup();
    }
    Ok(boards)
}

pub async fn fetch(
    http: &Http,
    config: &DeparturesConfig,
) -> std::result::Result<Vec<Board>, HttpError> {
    let mut boards = Vec::new();
    for url in monitor_urls(config) {
        let bytes = http.get("departures", url).await?;
        boards.extend(parse_monitor(&bytes).map_err(HttpError::from)?);
    }
    Ok(boards)
}

pub fn parse_catalogue(bytes: &[u8]) -> Result<Vec<Stop>> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_reader(bytes);
    let mut stops: Vec<Stop> = reader
        .deserialize()
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| "Invalid stop catalogue")?;
    if stops.is_empty()
        || stops
            .iter()
            .any(|stop| stop.diva == 0 || stop.name.trim().is_empty())
    {
        return Err("Stop catalogue is empty or invalid".into());
    }
    stops.sort_by_key(|stop| stop.diva);
    stops.dedup_by_key(|stop| stop.diva);
    stops.sort_by(|a, b| a.name.cmp(&b.name).then(a.diva.cmp(&b.diva)));
    Ok(stops)
}

pub async fn catalogue(http: &Http) -> std::result::Result<Vec<Stop>, HttpError> {
    parse_catalogue(
        &http
            .get("stop catalogue", CATALOGUE_URL.parse().unwrap())
            .await?,
    )
    .map_err(Into::into)
}

pub fn upcoming<'a>(board: &'a Board, lines: &[String], now: DateTime<Utc>) -> Vec<&'a Departure> {
    board
        .departures
        .iter()
        .filter(|departure| {
            departure.at >= now && (lines.is_empty() || line_selected(lines, &departure.line))
        })
        .collect()
}

pub fn line_selected(lines: &[String], name: &str) -> bool {
    lines
        .iter()
        .any(|line| line.trim().eq_ignore_ascii_case(name.trim()))
}

pub fn board_for_stop(boards: &[Board], stop: &StopConfig) -> Option<Board> {
    let mut matching = boards.iter().filter(|board| match stop.kind {
        StopKind::StopId => board.id == stop.id,
        StopKind::Diva => board.diva.parse::<u32>().ok() == Some(stop.id),
    });
    let mut combined = matching.next()?.clone();
    for board in matching {
        combined.departures.extend(board.departures.clone());
        combined
            .available_lines
            .extend(board.available_lines.clone());
    }
    combined.departures.sort_by(|a, b| {
        (a.at, &a.line, &a.direction, &a.destination, a.planned).cmp(&(
            b.at,
            &b.line,
            &b.direction,
            &b.destination,
            b.planned,
        ))
    });
    combined.departures.dedup_by(|a, b| {
        a.at == b.at
            && a.line == b.line
            && a.direction == b.direction
            && a.destination == b.destination
    });
    combined.available_lines.sort();
    combined.available_lines.dedup();
    Some(combined)
}

pub fn countdown(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let minutes = (at - now).num_seconds() / 60;
    if minutes < 1 {
        "*".into()
    } else {
        minutes.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_and_ids() {
        let stops =
            parse_catalogue(include_bytes!("../../../tests/fixtures/wl-stations.csv")).unwrap();
        assert!(
            stops
                .iter()
                .any(|stop| stop.diva == 60200267 && stop.name == "Eichenstraße")
        );
        assert_eq!(stops.len(), 2);
        assert!(parse_catalogue(b"wrong\nheader\n").is_err());
        assert!(parse_catalogue(b"DIVA;PlatformText\n0;Invalid\n").is_err());
        let config = DeparturesConfig {
            stops: vec![
                crate::config::StopConfig {
                    id: 2,
                    kind: StopKind::StopId,
                    label: "".into(),
                    lines: vec![]
                };
                2
            ],
            ..Default::default()
        };
        assert_eq!(monitor_urls(&config)[0].query(), Some("stopId=2"));
    }

    #[test]
    fn diva_requests_and_platforms_share_one_case_insensitive_line_filter() {
        let boards = parse_monitor(include_bytes!(
            "../../../tests/fixtures/wl-diva-monitor.json"
        ))
        .unwrap();
        let stop = StopConfig {
            id: 60200267,
            kind: StopKind::Diva,
            label: String::new(),
            lines: vec!["12a".into()],
        };
        let combined = board_for_stop(&boards, &stop).unwrap();
        assert_eq!(combined.available_lines, vec!["12A", "14A"]);
        assert_eq!(combined.departures.len(), 4);
        let now = DateTime::parse_from_rfc3339("2026-10-11T18:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let filtered = upcoming(&combined, &stop.lines, now);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|departure| departure.line == "12A"));
        assert_eq!(upcoming(&combined, &[], now).len(), 4);
        assert!(upcoming(&combined, &["12".into()], now).is_empty());
        assert!(line_selected(&stop.lines, "12A"));
        assert!(!line_selected(&stop.lines, "14A"));
        let config = DeparturesConfig {
            stops: vec![stop.clone(), stop],
            ..Default::default()
        };
        assert_eq!(monitor_urls(&config).len(), 1);
        assert_eq!(monitor_urls(&config)[0].query(), Some("diva=60200267"));
        let stopping_point = StopConfig {
            id: 399,
            kind: StopKind::StopId,
            label: String::new(),
            lines: vec![],
        };
        assert_eq!(
            board_for_stop(&boards, &stopping_point)
                .unwrap()
                .departures
                .len(),
            2
        );
        let mut overlapping = boards.clone();
        overlapping.extend(boards);
        assert_eq!(
            board_for_stop(&overlapping, &config.stops[0])
                .unwrap()
                .departures
                .len(),
            4
        );
    }

    #[test]
    fn filtering_does_not_mutate_and_countdown_changes_locally() {
        let now = Utc::now();
        let board = Board {
            id: 1,
            diva: "1".into(),
            title: "Test".into(),
            available_lines: vec![],
            departures: vec![Departure {
                line: "18".into(),
                destination: "Stadion".into(),
                direction: "1".into(),
                at: now + chrono::Duration::seconds(125),
                planned: true,
            }],
        };
        assert_eq!(countdown(board.departures[0].at, now), "2");
        assert_eq!(
            countdown(board.departures[0].at, now + chrono::Duration::seconds(100)),
            "*"
        );
        assert!(upcoming(&board, &["6".into()], now).is_empty());
        assert!(upcoming(&board, &[], now + chrono::Duration::seconds(126)).is_empty());
        assert_eq!(board.departures.len(), 1);
        let mut board = board;
        let mut later = board.departures[0].clone();
        later.at += chrono::Duration::minutes(3);
        board.departures.push(later);
        assert_eq!(display_departures(&board, &[], now, None).len(), 1);
        let mut selection = Selection {
            stop_id: 1,
            line: "18".into(),
            direction: "1".into(),
        };
        assert_eq!(
            display_departures(&board, &[], now, Some(&selection)).len(),
            2
        );
        selection.direction = "2".into();
        assert!(display_departures(&board, &[], now, Some(&selection)).is_empty());
    }
}
