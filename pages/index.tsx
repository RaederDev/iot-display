import type {NextPage} from 'next'
import Layout from '../components/layout';
import Greeting from '../components/applets/greeting';
import Weather from '../components/applets/weather';
import Departures from '../components/applets/departures/departures';
import BigClock from '../components/applets/big-clock';

const Home: NextPage = () => {
    return (
        <Layout>
            <Greeting />
            <div className="grid grid-cols-12">
                <div className="col-span-5">
                    <Weather />
                    <div className="mt-5"><BigClock /></div>
                </div>
                <div className="col-span-7 pl-8">
                    <Departures />
                </div>
            </div>
        </Layout>
    )
}

export default Home
