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

export const lineToDisplayLine = (line: Line): DisplayLine => {
    const departureTime = new Date(line.departures.departure[0].departureTime.timeReal);
    let minutes: string | number = differenceInMinutes(departureTime, new Date());
    if (minutes < 1) {
        minutes = '*';
    }

    return {
        name: line.name,
        destination: line.towards.toUpperCase(),
        time: minutes.toString(),
    };
};

export const removeAlreadyDeparted = (line: Line) => {
    const now = Date.now();
    line.departures.departure = line.departures
        .departure
        .filter(departure => (new Date(departure.departureTime.timeReal)).getTime() > now);
};

export const getCurrentLocationStops = (monitorData: MonitorResponse): Array<LocationStop> => {
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
                        lineToDisplayLine(line)
                    );
                }
            });
    });

    return Object.values(locationStops)
        .filter(stop => stop.displayLines.length > 0);
};
