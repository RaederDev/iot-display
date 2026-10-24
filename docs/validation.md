# Migration validation

Checked on 2026-10-11 with Rust 1.96 (the package requires Rust 1.95 or newer).

| Check | Result |
| --- | --- |
| Formatting and diff whitespace | Passed |
| Clippy, all targets, warnings denied | Passed on x86_64 and aarch64 Linux |
| Unit and integration tests | 28 passed on x86_64; 28 passed on aarch64 under QEMU |
| Release builds | Passed for x86_64 and aarch64 Linux |
| ARM executable startup | `--help` passed under QEMU with the cross-build sysroot |
| Settings interaction | Headless egui tests at 1024×600 and 1920×1080 cover Save/Cancel, persistence, invalid-file replacement, tabs, previews, dark styling, and keyboard access |
| Native window | Checked in KDE Wayland: first-run settings, windowed dashboard, and fullscreen dashboard |
| Live services | Open-Meteo forecast/geocoding and Wiener Linien monitor/catalogue returned the expected data |
| Fresh-cache restart | Native dashboard loaded weather/departures without printing HTTP request lines |
| Offline startup | With expired caches and an unreachable proxy, native fullscreen showed retained data, age, errors, and retry countdowns |
| Historical documentation | Sensor routine and service compared verbatim with the recorded source commit |
| DIVA catalogue update | Parser fixture uses rows from the downloaded `wienerlinien-ogd-haltestellen.csv`; a live DIVA monitor request returned Eichenstraße's platforms and lines |
| Departure settings regression checks | Catalogue selection stores DIVA numbers; typing `12a` updates the filter; platform aggregation, case-insensitive filtering, sorted request identities, and existing stopping-point configurations are covered |
| Dashboard text regression checks | Save clears its notice; dashboard attribution is absent, settings attribution remains, and catalogue refresh buttons and the old polling explanation are absent |
| Reference layout regression checks | Rendered text bounds at 800×600, 1024×600, and 1920×1080 confirm centered station headings, smaller departure text, right-aligned countdowns, destination ellipses without overlap, aligned Pi-hole statistics, and clickable departure rows/detail navigation |

The tests use a local HTTP server and captured subprocess stdout for authentication
renewal, session release, redirect logging, secret redaction, cache-hit silence,
timeouts, rate limits, deduplication, forced refresh, stale recovery, draft tests,
and cancellation after configuration changes. QEMU subprocess tests use
`IOT_DISPLAY_TEST_RUNNER`, an executable wrapper that runs the supplied test binary
with the appropriate ARM loader/sysroot; normal tests do not require this variable.

The weather selection follow-up passed the expanded 29-test suite and Clippy on
x86_64. Its headless UI test covers selection feedback, cleared searches, manual
coordinate updates, Clear, Save/Cancel, and persistence at both settings sizes.
The 28-test ARM results above predate this follow-up; its ARM build was not rerun.

The module cleanup on 2026-10-12 passed formatting, Clippy with warnings denied,
and release builds on x86_64 and aarch64. The expanded 30-test suite passed on
x86_64 and aarch64 under QEMU, including the weather selection follow-up. It also
checks departure overview/detail grouping, Pi-hole reconfiguration, layout draft
edits and Cancel, and rejecting search results after selection clears the query.
ARM executable `--help` passed under QEMU. Native window and live-service checks
in the table were performed during the earlier migration, not repeated for this
cleanup. See [code structure and event flow](architecture.md) for the new module
boundaries.

The timeout mock gives request transmission a 200 ms budget and delays its
response for 500 ms. It asserts that the server received the HTTP request before
the timeout; the earlier 20 ms budget was unreliable under ARM emulation.
Production request timeouts remain unchanged.

The widget registry refactor on 2026-10-12 passed formatting, Clippy with warnings
denied, and release builds on x86_64 and aarch64 Linux. All 39 tests passed on
x86_64 and aarch64 under QEMU. New checks cover unique registrations/provider
ownership, fixed refresh periods and removal of old JSON interval fields,
monotonic deadlines, cache loss and failed persistence, queued tests, minimum
gaps across cancellation/configuration changes, and oversized server cooldowns.
A mock HTTP test confirms that Save followed by Clear cache and repeated Refresh
clicks preserves pacing and fetches shared Pi-hole data once. Existing headless
UI checks still cover previews, widget settings, layouts, departure navigation,
and weather location selection. Both release executables passed `--help`;
physical Pi/labwc and manual desktop checks were not repeated for this refactor.

The ARM cross build used a GNU Linux sysroot and vendored OpenSSL. Its binary is
at `target/cross/aarch64-unknown-linux-gnu/release/iot-display`; the native binary
is at `target/release/iot-display`. Those generated files are ignored by Git.
Use a matching sysroot or build directly on the target Pi before deployment.

No physical Raspberry Pi, labwc session, EP-0106 sensor, or live Pi-hole installation
was used for the earlier migration validation. ARM emulation and KDE Wayland do
not verify Pi GPU drivers, touch hardware, or sustained operation. Complete the
hardware acceptance steps in the README on Pi 4/5 before claiming hardware support
has been verified.

## Automatic screen power — 2026-10-17

The native controller adds optional presence/input-driven screen sleep and wake,
defaulting off. Formatting, diff whitespace, Clippy for all targets with warnings
denied, and the native release build passed. All **53 native tests** passed
(43 library tests and 10 networking tests).

The aarch64 release build also passed using the existing
`ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main` image, GNU cross compiler, and
vendored OpenSSL. Its executable passed `--help` under QEMU with the image's
ARM sysroot. The new tests were run natively; the ARM test suite was not rerun.

New checks cover old/new configuration defaults and Save/Cancel persistence,
settings notices while disabled at 1024×600 and 1920×1080, three consecutive
sensor-only sleep/wake cycles, continuous presence during cooldown, exact idle
and cooldown boundaries, input/inhibitor gating, unavailable/recovery grace,
coalesced requests, partial/late acknowledgements, restoration ownership, and
discarding a late result from a single outstanding timed-out sensor reader.

An in-process Wayland server exercises the actual adapter: initial power-state
discovery, the ten-minute idle subscription, idle/resume events, acknowledged
power changes, leaving externally blanked outputs alone during restoration,
output hotplug/removal, rejected power control, missing idle/power protocols,
and a missing acknowledgement followed by bounded restoration waiting. It does
not modify the developer's desktop power state.

The earlier read-only inspection of `192.168.1.129` confirmed labwc 0.20.1,
an active `swayidle` ten-minute blanking rule, and a working `wlopm --json` query
reporting HDMI-A-1 off. I²C is disabled and `/dev/i2c-1` is missing. No server
configuration, packages, services, or running processes were changed. Thus board
identity/readings, wiring/calibration, real panel acknowledgements, and physical
keyboard/touch wake remain unverified. The README lists the required physical
acceptance scenarios after enabling I²C and removing competing blanking.
