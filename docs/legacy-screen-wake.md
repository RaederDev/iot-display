# Historical screen wake and sensor integration

This is a historical reference. The native application now implements optional
sensor-driven screen wake and sleep; see the [current setup instructions](../README.md#automatic-screen-wake-and-sleep).
The snippets below are not installation instructions.

Source commit: `c057dfb8087d52512c6027a10695aba955f3b8eb`. Original files: `bin/human-monitoring.ts`,
`bin/launch-human-monitoring.sh`, `bin/system/iot-human-monitoring@.service`,
`package.json`, and `README.md`.

## Hardware and access

The old installation used an EP-0106 Sensor Hub Development board. I²C had to be
enabled through raspi-config, and the graphical-session user needed membership
in the i2c group. The code opened I²C bus **1** (`/dev/i2c-1`), device address
**0x17**, and read register **0x0D**. Only the byte value **1** meant human presence.
Physical wiring and sensor calibration were not documented in the repository.

## Detection and screen behavior

A one-second timer lazily opened the bus and read the sensor. On detection the
code set a suppression flag, scheduled its reset after five minutes, and invoked
`xset dpms force on`. During those five minutes it skipped sensor reads completely.
The flag did not measure actual screen power state.

The application never turned the screen off or configured idle timeouts. Those
were supplied by the X11/Raspberry Pi environment. The ten-minute screen-on
period mentioned in the comment was an assumption, not enforced behavior.

Original detection routine, verbatim:

```typescript
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
```

## Historical launch setup

The shell launcher ran `npm run start-monitoring`; package.json mapped that to
`ts-node bin/human-monitoring.ts`. The systemd template ran under the X11-session
user, from /opt/iot-display, with DISPLAY=:0 and that user's .Xauthority file.
Restart=no meant the service would not automatically recover from a crash.
The old setup symlinked this unit into /lib/systemd/system and used a user-specific
`iot-human-monitoring@USERNAME` instance. Do not install this unit for the native app.

Original unit, verbatim:

```ini
[Unit]
Description=iot-display
Documentation=https://github.com/RaederDev/iot-display
After=graphical.target

[Service]
Type=simple
User=%I
ExecStart=/opt/iot-display/bin/launch-human-monitoring.sh
WorkingDirectory=/opt/iot-display/
Restart=no
Environment=DISPLAY=:0
Environment=XAUTHORITY=/home/%I/.Xauthority

[Install]
WantedBy=multi-user.target
```

## Known limitations

- I²C open/read errors were unhandled; timer callbacks could overlap while awaiting I/O.
- The bus was never explicitly closed.
- The five-minute suppression remained active even if xset failed.
- Error handling only printed xset errors; it did not verify screen power.
- No explicit screen-off policy, wiring specification, or calibration data existed.

## Native Wayland implementation

`src/screen_power/` separates sensor acquisition, monotonic policy, and Wayland
display control. It reads the same register through `i2cdev` and uses
`ext-idle-notify-v1` plus `wlr-output-power-management-v1`, without X11 commands or
an external idle daemon. Settings opt in to the controller and report unavailable
hardware/display support. A read-only availability probe also runs when disabled.

The native controller owns both sleep and wake. It combines session-wide input
idleness with the latest positive presence reading and turns outputs off after
ten minutes without either. The five-minute cooldown starts only after a
confirmed sensor wake; polling continues during cooldown, and input wake bypasses
it. Every cycle rearms through the controller's own presence deadline, avoiding
the old assumption that powering on automatically restarts desktop idle timers.

Failed or disconnected hardware suspends automatic blanking and triggers
best-effort restoration of app-blanked outputs. Device reads and compositor
acknowledgements have supervisory deadlines; only one sensor read can remain
outstanding. Physical wiring, calibration, register behavior, and actual panel
wake still require acceptance on the attached EP-0106. The inspected server has
I²C disabled; implementation does not change that server's configuration.
