import React from 'react';

interface Props {
    theme?: 'black' | 'white';
    children: Array<JSX.Element> | JSX.Element;
}

export default function Card({theme, children}: Props) {
    const themeClass = theme === 'black' ? 'text-white bg-darkgray' : 'bg-white';
    return (
        <div className={'relative py-4 px-4 rounded-sm shadow-md ' + themeClass}>
            {children}
        </div>
    );
}
