import * as config from '../config/config.json';

export enum ConfigKey {
    USER = 'user',
    WL_STOPS = 'wlStops',
    NETWORK_INTERFACE = 'networkInterface',
    PI_HOLE_HOST = 'piHoleHost',
    OPEN_WEATHER = 'openWeather',
    LAYOUT = 'layout',
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

export interface LayoutConfigChild {
    width: number;
    widgets: Array<string>;
}

export interface LayoutConfig {
    top: LayoutConfigChild;
    left: LayoutConfigChild;
    right: LayoutConfigChild;
}

export function getConfigValue(key: ConfigKey): any {
    return config[key];
}
