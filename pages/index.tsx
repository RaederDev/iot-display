import type {NextPage} from 'next'
import Layout from '../components/layout';
import {ConfigKey, getConfigValue, LayoutConfig} from '../lib/config';
import {getWidget} from '../components/widgets/widgets';

const Home: NextPage = () => {
    const config: LayoutConfig = getConfigValue(ConfigKey.LAYOUT);

    return (
        <Layout>
            <div className="grid grid-cols-12">
                <div className={'col-span-' + config.top.width}>
                    {config.top.widgets.map((widget: string, i: number) => (
                        <div key={i}>{getWidget(widget)}</div>
                    ))}
                </div>
            </div>
            <div className="grid grid-cols-12">
                <div className={'col-span-' + config.left.width}>
                    {config.left.widgets.map((widget: string, i: number) => (
                        <div className="mt-5 first:mt-0" key={i}>{getWidget(widget)}</div>
                    ))}
                </div>
                <div className={'pl-5 col-span-' + config.right.width}>
                    {config.right.widgets.map((widget: string, i: number) => (
                        <div className="mt-5 first:mt-0" key={i}>{getWidget(widget)}</div>
                    ))}
                </div>
            </div>
        </Layout>
    )
}

export default Home;
