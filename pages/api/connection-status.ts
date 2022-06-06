import type {NextApiRequest, NextApiResponse} from 'next';
// @ts-ignore
import * as network from 'network';
import {ConfigKey, getConfigValue} from '../../lib/config';
import {ApiResponse} from '../../lib/response';

export interface ConnectionStatusData {
    ip?: string;
}

export interface ConnectionStatusResponse extends ApiResponse {
    data?: ConnectionStatusData;
}

const getIpForConfiguredInterface = (): Promise<string> => {
    const configuredInterface: string = getConfigValue(ConfigKey.NETWORK_INTERFACE);
    return new Promise((resolve, reject) => {
        network.get_interfaces_list((err: any, list: any) => {
            if (err) {
                reject(err);
                return;
            }

            const found = list.find((iface: any) => configuredInterface === iface.name);
            if (!found) {
                reject('Interface not found');
                return;
            }

            resolve(found.ip_address);
        });
    });
};

export default async function getConnectionStatus(req: NextApiRequest, res: NextApiResponse<ConnectionStatusResponse>) {
    try {
        const ip = await getIpForConfiguredInterface();
        return res.status(200).json({data: {ip}});
    } catch (e) {
        return res.status(500).json({error: 'Interface not found'});
    }
}
