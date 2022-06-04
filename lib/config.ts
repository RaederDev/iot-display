import * as config from '../config/config.json';

export enum ConfigKey {
    USER = 'user',
    WL_STOPS = 'wlStops',
    NETWORK_INTERFACE = 'networkInterface',
    PI_HOLE_HOST = 'piHoleHost',
    OPEN_WEATHER = 'openWeather',
}

export interface WlStopConfig {
    stopId: number;
    diva: string;
    lines: Array<string>;
}

export interface OpenWeatherConfig {
    apiKey: string;
    country: string;
    zipCode: number;
    unit: 'metric' | 'imperial';
}

export function getConfigValue(key: ConfigKey): any {
    return config[key];
}
