import type {NextApiRequest, NextApiResponse} from 'next';
import {ApiResponse} from '../../lib/response';

export interface PiHoleResponse {
    ads_blocked_today: number;
    ads_percentage_today: number;
    dns_queries_today: number;
}

const API_URL = 'http://pi.hole/admin/api.php';

export default async function getWeather(req: NextApiRequest, res: NextApiResponse<ApiResponse<PiHoleResponse>>) {
    try {
        const result: PiHoleResponse = await (await fetch(API_URL)).json();
        return res.status(200).json({data: result});
    } catch (e) {
        return res.status(500).json({error: 'internal error'});
    }
}
