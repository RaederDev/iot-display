import {DisplayLine, LocationStop} from './departures.helpers';

interface DepartureTableProps {
    locationStop: LocationStop;
    onDepartureTapped?: (locationStop: LocationStop, line: DisplayLine) => any;
}

export default function DepartureTable({locationStop, onDepartureTapped}: DepartureTableProps) {
    function click(locationStop: LocationStop, line: DisplayLine) {
        if (!onDepartureTapped) {
            return () => {};
        }

        return onDepartureTapped.bind(null, locationStop, line);
    }

    return <table className="w-full text-neongreen font-dseg mb-5 last:mb-0">
        <thead>
        <tr>
            <td colSpan={3} className="text-center pb-2">
                {locationStop.title.toUpperCase()}
            </td>
        </tr>
        </thead>
        <tbody>
        {locationStop.displayLines?.map((line, id) => (
            <tr key={id} className="h-8" onClick={click(locationStop, line)}>
                <td className="text-left">{line.name}</td>
                <td className="text-left w-full px-4">{line.destination}</td>
                <td className="text-right">{line.time}</td>
            </tr>
        ))}
        </tbody>
    </table>;
}
