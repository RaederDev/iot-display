import {useEffect, useState} from 'react';
import Card from '../card';
import {format} from 'date-fns';

export default function BigClock() {
    const [date, setDate] = useState('');
    const update = () => setDate(format(new Date(), 'HH:mm'));

    useEffect(() => {
        update();
        const interval = setInterval(update, 1000);
        return () => clearInterval(interval);
    }, []);

    return (
        <Card>
            <div className="text-6xl text-center">{date}</div>
        </Card>
    );
}
