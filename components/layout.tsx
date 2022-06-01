import Head from 'next/head';
import React from 'react';
import StatusBar from './status-bar';

export default function Layout({children}: {children: Array<JSX.Element> | JSX.Element}) {
    return (
        <div className="box-border">
            <Head>
                <title>IOT Display</title>
                <meta httpEquiv="X-UA-Compatible" content="IE=edge"/>
                <meta charSet="utf-8"/>
                <link rel="shortcut icon" href="/favicon.ico"/>
            </Head>
            <main className="font-sans bg-lightgray min-h-screen">
                <StatusBar />
                <div className="py-5 px-5">{children}</div>
            </main>
        </div>
    );
}
