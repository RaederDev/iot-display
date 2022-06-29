import Greeting from './greeting';
import Departures from './departures/departures';
import Weather from './weather';
import PiHole from './pi-hole';
import React from 'react';

const widgets: Record<string, () => JSX.Element> = {
    'greeting': Greeting,
    'departures': Departures,
    'weather': Weather,
    'pi-hole': PiHole,
};

export function getWidget(name: string): JSX.Element | null {
    if (!widgets[name]) {
        return null;
    }

    return React.createElement(widgets[name]);
}
