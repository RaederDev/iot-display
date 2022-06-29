import type {NextPage} from 'next'
import Layout from '../components/layout';
import Greeting from '../components/widgets/greeting';
import Weather from '../components/widgets/weather';
import Departures from '../components/widgets/departures/departures';
import BigClock from '../components/widgets/big-clock';
import PiHole from '../components/widgets/pi-hole';

const Home: NextPage = () => {
    return (
        <Layout>
            <Greeting />
            <div className="grid grid-cols-12">
                <div className="col-span-5">
                    <Weather />
                    <div className="mt-5"><PiHole /></div>
                </div>
                <div className="col-span-7 pl-8">
                    <Departures />
                </div>
            </div>
        </Layout>
    )
}

export default Home
