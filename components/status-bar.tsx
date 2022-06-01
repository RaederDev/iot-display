import Clock from './applets/clock';
import ConnectionStatus from './applets/connection-status';

export default function StatusBar() {
    return (
        <div className='flex justify-between px-3 py-1 bg-blue text-white '>
            <ConnectionStatus />
            <Clock />
        </div>
    );
}
