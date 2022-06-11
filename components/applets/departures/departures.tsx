import {ConfigKey, getConfigValue, WlStopConfig} from '../../../lib/config';
import Card from '../../card';
import {useEffect, useRef, useState} from 'react';
import {CACHE_KEY, getCachedObject, setCachedObject} from '../../../lib/cache';
import {MonitorResponse} from '../../../pages/api/wl-realtime-monitor/[id]';
import {DisplayLine, getAllLocationStopsForLine, getCurrentLocationStops, LocationStop} from './departures.helpers';
import DepartureTable from './departure-table';
import {IoArrowBack} from '@react-icons/all-files/io5/IoArrowBack';

interface DetailView {
    locationStop: LocationStop;
    line: DisplayLine;
}

export default function Departures() {
    const [locationStops, setLocationStops] = useState<Array<LocationStop>>([]);
    const [detailLocationStops, setDetailLocationStops] = useState<Array<LocationStop>>([]);
    const [detailView, setDetailView] = useState<DetailView>();
    const stopsConfig = useRef<Array<WlStopConfig>>(getConfigValue(ConfigKey.WL_STOPS));
    const monitorData = useRef<MonitorResponse>();

    useEffect(() => {
        const updateDisplayLines = () => {
            if (monitorData.current) {
                setLocationStops(getCurrentLocationStops(monitorData.current));
            }
        };

        const refreshData = async () => {
            const cachedResponse = getCachedObject<MonitorResponse>(CACHE_KEY.DEPARTURES_RESPONSE, 60);
            console.log('cached departures data', cachedResponse);

            if (cachedResponse) {
                monitorData.current = cachedResponse;
                updateDisplayLines();
                return;
            }

            const ids = stopsConfig.current.map(stop => stop.stopId).join(',');
            monitorData.current = await (await fetch(`api/wl-realtime-monitor/${ids}`)).json();
            setCachedObject(CACHE_KEY.DEPARTURES_RESPONSE, monitorData.current);
            updateDisplayLines();
            console.log('fetched departures', monitorData.current);
        };

        refreshData();
        const apiFetchInterval = setInterval(refreshData, 1000 * 60 * 5);
        const uiRefreshInterval = setInterval(updateDisplayLines, 1000 * 5);

        return () => {
            clearInterval(apiFetchInterval);
            clearInterval(uiRefreshInterval);
        };
    }, []);

    useEffect(() => {
        if (!detailView) {
            return;
        }

        const refresh = () => {
            const stops = getAllLocationStopsForLine(
                monitorData.current,
                detailView.locationStop,
                detailView.line,
            );
            setDetailLocationStops(stops);
        };
        refresh();
        const detailViewRefreshInterval = setInterval(refresh, 1000 * 5);

        return () => clearInterval(detailViewRefreshInterval);
    }, [detailView, monitorData]);

    function onDepartureTapped(locationStop: LocationStop, line: DisplayLine) {
        setDetailView({
            locationStop,
            line,
        });
    }

    function closeDetailView() {
        setDetailView(undefined);
    }

    function renderNoDepartures() {
        return <div className="text-center text-neongreen font-dseg my-5">
            ***&nbsp;&nbsp;&nbsp;NO DEPARTURES&nbsp;&nbsp;&nbsp;***
        </div>;
    }

    function renderAllDeparturesTable(stops: Array<LocationStop>) {
        return stops.map((stop: LocationStop, i: number) =>
            <DepartureTable key={i} onDepartureTapped={onDepartureTapped} locationStop={stop}/>,
        );
    }

    function render() {
        if (locationStops.length < 1) {
            return renderNoDepartures();
        }

        if (detailView) {
            return <>
                <IoArrowBack onClick={closeDetailView} className="absolute text-neongreen"/>
                {detailLocationStops.length > 0 ?
                    renderAllDeparturesTable(detailLocationStops) :
                    renderNoDepartures()
                }
            </>;
        }
        return renderAllDeparturesTable(locationStops);
    }

    return <Card theme="black">{render()}</Card>;
}
