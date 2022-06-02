import {ConfigKey, getConfigValue, WlStopConfig} from '../../../lib/config';
import Card from '../../card';
import {useEffect, useState} from 'react';
import {CACHE_KEY, getCachedObject, setCachedObject} from '../../../lib/cache';
import {MonitorResponse} from '../../../pages/api/wl-realtime-monitor/[id]';
import {getCurrentLocationStops, LocationStop} from './departures.helpers';

export default function Departures() {
    const [locationStops, setLocationStops] = useState<Array<LocationStop>>([]);

    useEffect(() => {
        const stopsConfig: Array<WlStopConfig> = getConfigValue(ConfigKey.WL_STOPS);
        let monitorData: MonitorResponse | null = null;

        const updateDisplayLines = () => {
            if (!monitorData) {
                return;
            }

            setLocationStops(getCurrentLocationStops(monitorData));
        };

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

        refreshData();
        const apiFetchInterval = setInterval(refreshData, 1000 * 60 * 5);
        const uiRefreshInterval = setInterval(updateDisplayLines, 1000 * 5);

        return () => {
            clearInterval(apiFetchInterval);
            clearInterval(uiRefreshInterval);
        };
    }, []);

    return (
        locationStops.length > 0 ?
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
            </Card>:
            <Card theme="black">
                <div className="text-center text-neongreen font-dseg my-5">
                    ***&nbsp;&nbsp;&nbsp;NO DEPARTURES&nbsp;&nbsp;&nbsp;***
                </div>
            </Card>
    );
}
