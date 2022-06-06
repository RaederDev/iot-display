import type {NextApiRequest, NextApiResponse} from 'next';
import {ApiResponse} from '../../lib/response';
import {ConfigKey, getConfigValue} from '../../lib/config';

export interface PiHoleData {
    ads_blocked_today: number;
    ads_percentage_today: number;
    dns_queries_today: number;
    unique_domains: number;
}

export interface PiHoleResponse extends ApiResponse {
    data?: PiHoleData;
}

const API_URL = `http://${getConfigValue(ConfigKey.PI_HOLE_HOST)}/admin/api.php`;

export default async function getStats(req: NextApiRequest, res: NextApiResponse<PiHoleResponse>) {
    try {
        const result: PiHoleData = await (await fetch(API_URL)).json();
        return res.status(200).json({data: result});
    } catch (e) {
        console.error(e);
        return res.status(500).json({error: 'internal error'});
    }
}
