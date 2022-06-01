import {useEffect, useState} from 'react';
import {format} from 'date-fns';

export default function Clock() {
    const [date, setDate] = useState('');
    const update = () => setDate(format(new Date(), 'dd.MM.yyyy HH:mm:ss'));

    useEffect(() => {
        update();
        const interval = setInterval(update, 1000);
        return () => clearInterval(interval);
    }, []);

    return <span>{date}</span>;
}
