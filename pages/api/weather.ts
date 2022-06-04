import type {NextApiRequest, NextApiResponse} from 'next';
// @ts-ignore
import {AsyncWeather} from '@cicciosgamino/openweather-apis';
import {ConfigKey, getConfigValue, OpenWeatherConfig} from '../../lib/config';

type WeatherConditionMain = 'Thunderstorm' | 'Drizzle' | 'Rain'
    | 'Snow' | 'Atmosphere' | 'Clear' | 'Clouds';

interface Coords {
    lon: number;
    lat: number;
}

interface Weather {
    id: number;
    main: WeatherConditionMain;
    description: string;
    icon: string;
}

interface Main {
    temp: number;
    feels_like: number;
    temp_min: number;
    temp_max: number;
    pressure: number;
    humidity: number;
}

export interface WeatherResponse {
    coord: Coords;
    weather: Array<Weather>;
    name: string;
    main: Main;
}

export default async function getWeather(req: NextApiRequest, res: NextApiResponse<WeatherResponse>) {
    const weatherConfig: OpenWeatherConfig = getConfigValue(ConfigKey.OPEN_WEATHER);
    if (!weatherConfig.apiKey) {
        throw new Error('missing api key');
    }

    const weather = await new AsyncWeather();
    weather.setLang('en');
    weather.setZipCodeAndCountryCode(weatherConfig.zipCode, weatherConfig.country);
    weather.setUnits(weatherConfig.unit);
    weather.setApiKey(weatherConfig.apiKey);

    try {
        const resp: WeatherResponse = await weather.getAllWeather();
        resp.weather = resp.weather.map(weather => {
            weather.icon = `https://openweathermap.org/img/wn/${weather.icon}@2x.png`;
            return weather;
        });
        return res.status(200).json(resp);
    } catch (e) {
        return res.status(500);
    }
}
