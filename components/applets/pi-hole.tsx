import {useEffect, useState} from 'react';
import Card from '../card';
import {PiHoleResponse} from '../../pages/api/pi-hole';
import Image from 'next/image';

export default function PiHole() {
    const [response, setResponse] = useState<PiHoleResponse>();

    useEffect(() => {
        const update = () => fetch('api/pi-hole')
            .then(res => res.json())
            .then(setResponse);
        update();
        const interval = setInterval(update, 1000 * 60);
        return () => clearInterval(interval);
    }, []);

    return (
        <Card>
            <div className="text-center mb-5">
                <Image src='/assets/img/pi-hole.svg' width={70} height={70} alt="pi-hole logo" />
            </div>
            <table className="w-full text-xl">
                <tbody>
                    <tr>
                        <td>DNS queries today</td>
                        <td>{response?.data?.dns_queries_today}</td>
                    </tr>
                    <tr>
                        <td>Ads blocked today</td>
                        <td>{response?.data?.ads_blocked_today}</td>
                    </tr>
                    <tr>
                        <td>Unique Domains</td>
                        <td>{response?.data?.unique_domains}</td>
                    </tr>
                </tbody>
            </table>
        </Card>
    );
}
