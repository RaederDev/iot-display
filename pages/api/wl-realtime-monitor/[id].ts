import type {NextApiRequest, NextApiResponse} from 'next';
import {ApiResponse} from '../../../lib/response';

const API_BASE_URL = 'https://www.wienerlinien.at/ogd_realtime/';

export interface DepartureTime {
    timePlanned: string;
    timeReal: string;
    countdown: number;
}

export interface Departure {
    departureTime: DepartureTime;
}

export interface Departures {
    departure: Array<Departure>;
}

export interface Line {
    name: string;
    towards: string;
    direction: string;
    platform: string;
    richtungsId: string;
    barrierFree: boolean;
    realtimeSupported: boolean;
    trafficjam: boolean;
    departures: Departures;
    type: string;
    lineId: number;
}

export interface LocationStopProperties {
    name: string;
    title: string;
    municipality: string;
    municipalityId: number;
    type: string;
    coordName: string;
}

export interface LocationStop {
    type: string;
    properties: LocationStopProperties;
}

export interface Monitor {
    locationStop: LocationStop;
    lines: Array<Line>;
}

export interface Data {
    monitors: Array<Monitor>;
}

export interface Message {
    value: string;
    messageCode: number;
    serverTime: string;
}

export interface MonitorResponse {
    data?: Data;
    message?: Message;
}

export default async function getMonitor(req: NextApiRequest, res: NextApiResponse<ApiResponse<MonitorResponse>>) {
    try {
        const query = (req.query.id as string).split(',')
            .map(id => `stopId=${id}`)
            .join('&');
        const reqUrl = `${API_BASE_URL}monitor?${query}`;
        const result: MonitorResponse = await (await fetch(reqUrl)).json();
        if (result?.message?.value !== 'OK') {
            console.error(result);
            return res.status(500).json({error: 'internal error'});
        }
        return res.status(200).json({data: result});
    } catch (e) {
        return res.status(500).json({error: 'internal error'});
    }
}
