import {ConfigKey, getConfigValue} from '../../lib/config';

export default function Greeting() {
    return (<div className='text-2xl mx-3 my-5'>
        Hello {getConfigValue(ConfigKey.USER)}, here&apos;s today&apos;s briefing:
    </div>);
}
