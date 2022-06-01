const CACHE_PREFIX = 'cache_';

export interface CacheEntry {
    cachedTime: number;
    content: string;
}

export enum CACHE_KEY {
    WEATHER_RESPONSE = 'weatherResponse',
    DEPARTURES_RESPONSE = 'departuresResponse'
}

export function getCachedObject<T>(key: CACHE_KEY, validForSeconds: number = -1): T | null {
    const cached = localStorage.getItem(`${CACHE_PREFIX}${key}`);
    if (!cached) {
        return null;
    }

    const parsed: CacheEntry = JSON.parse(cached);
    if (validForSeconds < 0 || (parsed.cachedTime + (validForSeconds * 1000)) > Date.now()) {
        return JSON.parse(parsed.content) as T;
    }

    return null;
}

export function setCachedObject(key: CACHE_KEY, obj: any) {
    localStorage.setItem(`${CACHE_PREFIX}${key}`, JSON.stringify({
        cachedTime: Date.now(),
        content: JSON.stringify(obj),
    }));
}
