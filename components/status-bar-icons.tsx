import {IoReload} from '@react-icons/all-files/io5/IoReload';
import {IoClose} from '@react-icons/all-files/io5/IoClose';

export default function StatusBarIcons() {
    const close = () => location.href = 'http://localhost:3000/iot-close';
    const reload = () => location.reload();

    return (
        <ul className="flex">
            <li className="h-full flex items-center" onClick={reload}>
                <span className="px-2">
                    <IoReload />
                </span>
            </li>
            <li className="h-full flex items-center" onClick={close}>
                <span className="px-2">
                    <IoClose />
                </span>
            </li>
        </ul>
    );
}
