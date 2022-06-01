import {useEffect, useState} from 'react';
import {ConnectionStatusResponse} from '../../pages/api/connection-status';

export default function ConnectionStatus() {
    const [connectionData, setConnectionData] = useState<ConnectionStatusResponse>();

    const update = () => {
        fetch('api/connection-status')
            .then(res => res.json())
            .then(setConnectionData);
    };

    useEffect(() => {
        update();
        const interval = setInterval(update, 1000 * 60);
        return () => clearInterval(interval);
    }, []);

    return (
        <span>IP: {connectionData?.ip}</span>
    );
}
