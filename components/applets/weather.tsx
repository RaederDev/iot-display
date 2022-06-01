import {useEffect, useState} from 'react';
import {CACHE_KEY, getCachedObject, setCachedObject} from '../../lib/cache';
import {WeatherResponse} from '../../pages/api/weather';
import Card from '../card';
import {IoArrowUp} from '@react-icons/all-files/io5/IoArrowUp';
import {IoArrowDown} from '@react-icons/all-files/io5/IoArrowDown';

export default function Weather() {
    const [weatherData, setWeatherData] = useState<WeatherResponse>();
    const refreshData = () => {
        const cachedResponse = getCachedObject<WeatherResponse>(CACHE_KEY.WEATHER_RESPONSE, 60 * 60);
        console.log('cached weather data', cachedResponse);
        if (!cachedResponse) {
            fetch('api/weather')
                .then(res => res.json())
                .then(data => {
                    console.log('fetched weather data', data);
                    setCachedObject(CACHE_KEY.WEATHER_RESPONSE, data);
                    setWeatherData(data);
                });
        } else {
            setWeatherData(cachedResponse);
        }
    };

    useEffect(() => {
        refreshData();
        const interval = setInterval(refreshData, 1000 * 60 * 5);
        return () => clearInterval(interval);
    }, []);

    if (!weatherData) {
        return <Card>
            <div>Loading weather information...</div>
        </Card>;
    }

    return (
        <Card>
            <div className="flex justify-between items-center">
                <div>
                    <div className="text-3xl">{weatherData.name}</div>
                    <div className="text-4xl">{Math.round(weatherData.main.temp)}°C</div>
                    <div>
                        {weatherData.weather[0].description}.
                        feels like {Math.round(weatherData.main.feels_like)}°C
                    </div>
                    <div className="flex items-center">
                        high. {Math.round(weatherData.main.temp_max)}°C
                        <IoArrowUp/>
                    </div>
                    <div className="flex items-center">
                        low. {Math.round(weatherData.main.temp_min)}°C
                        <IoArrowDown/>
                    </div>
                </div>
                <div><img src={weatherData.weather[0].icon} alt="icon"/></div>
            </div>
        </Card>
    );
}
