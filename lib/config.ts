import * as config from '../config/config.json';

export enum ConfigKey {
    USER = 'user',
    OPEN_WEATHER_API_KEY = 'openWeatherApiKey',
    WL_STOPS = 'wlStops',
    NETWORK_INTERFACE = 'networkInterface',
}

export interface WlStopConfig {
    stopId: number;
    diva: string;
    lines: Array<string>;
}

export function getConfigValue(key: ConfigKey): any {
    return config[key];
}
