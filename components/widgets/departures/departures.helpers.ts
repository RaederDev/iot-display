import {Line, MonitorResponse} from '../../../pages/api/wl-realtime-monitor/[id]';
import {differenceInMinutes} from 'date-fns';
import {ConfigKey, getConfigValue, WlStopConfig} from '../../../lib/config';

export interface LocationStop {
    name: string;
    title: string;
    displayLines: Array<DisplayLine>;
}

export interface DisplayLine {
    name: string;
    destination: string;
    time: string;
}

export function departureTimeToMinutes(timeReal: string): string {
    const departureTime = new Date(timeReal);
    let minutes: string | number = differenceInMinutes(departureTime, new Date());
    if (minutes < 1) {
        return '*';
    }

    return minutes.toString();
}

export function lineToDisplayLine(line: Line): DisplayLine {
    return {
        name: line.name,
        destination: line.towards.toUpperCase(),
        time: departureTimeToMinutes(line.departures.departure[0].departureTime.timeReal),
    };
}

export function lineToDisplayLines(line: Line): Array<DisplayLine> {
    return line.departures.departure.map(departure => ({
        name: line.name,
        destination: line.towards.toUpperCase(),
        time: departureTimeToMinutes(departure.departureTime.timeReal),
    }));
}

export function removeAlreadyDeparted(line: Line) {
    const now = Date.now();
    line.departures.departure = line.departures
        .departure
        .filter(departure => (new Date(departure.departureTime.timeReal)).getTime() > now);
}

export function getCurrentLocationStops(monitorData: MonitorResponse): Array<LocationStop> {
    const stopsConfig: Array<WlStopConfig> = getConfigValue(ConfigKey.WL_STOPS);
    const locationStops: { [key: string]: LocationStop; } = {};

    monitorData?.data?.monitors.forEach(monitor => {
        const locationProps = monitor.locationStop.properties;
        const wlStopConfig = stopsConfig.find(cfg => cfg.diva === locationProps.name);
        if (!locationStops[locationProps.name]) {
            locationStops[locationProps.name] = {
                name: locationProps.name,
                title: locationProps.title,
                displayLines: [],
            };
        }

        monitor.lines
            .filter(line => wlStopConfig?.lines.includes(line.name))
            .forEach(line => {
                removeAlreadyDeparted(line);
                if (line.departures.departure.length > 0) {
                    locationStops[locationProps.name].displayLines.push(
                        lineToDisplayLine(line),
                    );
                }
            });
    });

    return Object.values(locationStops)
        .filter(stop => stop.displayLines.length > 0);
}

export function getAllLocationStopsForLine(
    monitorData: MonitorResponse|undefined,
    locationStop: LocationStop,
    displayLine: DisplayLine,
): Array<LocationStop> {
    const locationStops: Array<LocationStop> = [];
    monitorData?.data?.monitors
        .filter(monitor => monitor.locationStop.properties.name === locationStop.name)
        .forEach(monitor => {
            const locationProps = monitor.locationStop.properties;
            const stop: LocationStop = {
                name: locationProps.name,
                title: locationProps.title,
                displayLines: [],
            };

            monitor.lines
                .filter(line => displayLine.name === line.name)
                .forEach(line => {
                    removeAlreadyDeparted(line);
                    if (line.departures.departure.length > 0) {
                        stop.displayLines.push(...lineToDisplayLines(line));
                    }
                });

            if (stop.displayLines.length > 0) {
                locationStops.push(stop);
            }
        });

    return locationStops;
}
