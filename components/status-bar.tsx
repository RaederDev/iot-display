import Clock from './applets/clock';
import ConnectionStatus from './applets/connection-status';
import StatusBarIcons from './status-bar-icons';

export default function StatusBar() {
    return (
        <div className='flex justify-between px-3 py-1 bg-blue text-white '>
            <StatusBarIcons />
            <div className="flex">
                <ConnectionStatus />
                <div className="mx-3">
                    |
                </div>
                <Clock />
            </div>
        </div>
    );
}
