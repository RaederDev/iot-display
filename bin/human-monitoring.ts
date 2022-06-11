// @ts-ignore
const i2c = require('i2c-bus');
const {exec} = require("child_process");

const SENSORHUB_BOARD = 0x17;
const HUMAN_DETECT = 0x0D;
let bus: any;
// Flag to avoid constantly calling xset.
// Screen should stay on for 10 minutes in default rpi config
let dpmsOn: boolean = false;

async function detectHumans() {
    if (dpmsOn) {
        return;
    }

    if (!bus) {
        bus = await i2c.openPromisified(1);
    }

    const humanPresent = await bus.readByte(SENSORHUB_BOARD, HUMAN_DETECT) === 1;
    if (humanPresent) {
        dpmsOn = true;
        setTimeout(() => dpmsOn = false, 1000 * 60 * 5);
        exec('xset dpms force on', (error: any, _: any, stderr: any) => {
            if (error || stderr) {
                console.error(error, stderr);
            }
        });
    }
}

setInterval(detectHumans, 1000);
