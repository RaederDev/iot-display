#!/usr/bin/env node

const puppeteer = require('puppeteer');

(async () => {
    const browser = await puppeteer.launch({
        headless: false,
        args: [
            '--start-maximized',
            '--kiosk',
        ],
        ignoreDefaultArgs: [
            '--enable-automation',
        ]
    });
    const page = await browser.newPage();
    await page.setViewport({
        width: 1024,
        height: 600,
        hasTouch: true,
    });
    await page.goto('http://localhost:3000/');
    browser.on('targetchanged', ev => {
        if (ev.url() === 'http://localhost:3000/iot-close') {
            browser.close();
        }
    });
})();

