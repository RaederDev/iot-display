import {ConfigKey, getConfigValue, WlStopConfig} from '../../lib/config';
import Card from '../card';
import {useEffect, useState} from 'react';
import {CACHE_KEY, getCachedObject, setCachedObject} from '../../lib/cache';
import {Line, MonitorResponse} from '../../pages/api/wl-realtime-monitor/[id]';
import {differenceInMinutes} from 'date-fns';

interface LocationStop {
    name: string;
    title: string;
    displayLines: Array<DisplayLine>;
}

interface DisplayLine {
    name: string;
    destination: string;
    time: string;
}

export default function Departures() {
    const stopsConfig: Array<WlStopConfig> = getConfigValue(ConfigKey.WL_STOPS);
    const [locationStops, setLocationStops] = useState<Array<LocationStop>>([]);
    let monitorData: MonitorResponse | null = null;

    const refreshData = async () => {
        const cachedResponse = getCachedObject<MonitorResponse>(CACHE_KEY.DEPARTURES_RESPONSE, 60);
        console.log('cached departures data', cachedResponse);

        if (cachedResponse) {
            monitorData = cachedResponse;
            updateDisplayLines();
            return;
        }

        const ids = stopsConfig.map(stop => stop.stopId).join(',');
        const response = await (await fetch(`api/wl-realtime-monitor/${ids}`)).json();
        setCachedObject(CACHE_KEY.DEPARTURES_RESPONSE, response.data);
        monitorData = response.data;
        updateDisplayLines();
        console.log('fetched departures', response);
    };

    useEffect(() => {
        refreshData();
        const apiFetchInterval = setInterval(refreshData, 1000 * 60 * 5);
        const uiRefreshInterval = setInterval(updateDisplayLines, 1000 * 5);

        return () => {
            clearInterval(apiFetchInterval);
            clearInterval(uiRefreshInterval);
        };
    }, []);

    const getDisplayLine = (line: Line): DisplayLine => {
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

    const removeAlreadyDeparted = (line: Line) => {
        const now = Date.now();
        line.departures.departure = line.departures
            .departure
            .filter(departure => (new Date(departure.departureTime.timeReal)).getTime() > now);
    };

    const updateDisplayLines = () => {
        if (!monitorData) {
            return;
        }

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
                        locationStops[locationProps.name].displayLines.push(getDisplayLine(line));
                    }
                });
        });

        setLocationStops(Object.values(locationStops));
    };

    return (
        <Card theme="black">
            {locationStops.map((locationStop, il) => (
                <table key={il} className="w-full text-neongreen font-dseg mb-5 last:mb-0">
                    <thead>
                        <tr>
                            <td colSpan={3} className="text-center pb-2">
                                {locationStop.title.toUpperCase()}
                            </td>
                        </tr>
                    </thead>
                    <tbody>
                    {locationStop.displayLines?.map((line, id) => (
                        <tr key={id} className="h-8">
                            <td className="text-left">{line.name}</td>
                            <td className="text-left w-full px-4">{line.destination}</td>
                            <td className="text-right">{line.time}</td>
                        </tr>
                    ))}
                    </tbody>
                </table>
            ))}
        </Card>
    );
}
