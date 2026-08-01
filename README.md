# Sleep Enforcer

A bedtime enforcer for Linux. At a fixed time each night a fullscreen, always on
top countdown takes over the screen, and a few minutes later the machine powers
itself off.

The window has no decorations, no close button, and cannot be resized, so getting
rid of it means killing the process. That friction is deliberate, but the window
is not what enforces bedtime. The shutdown runs on a separate timer and fires
whether or not the countdown is on screen.

## How it works

Two systemd user timers, each triggering its own service:

| Unit | Fires | Does |
| --- | --- | --- |
| `sleep_enforcer.timer` | 00:15 | starts `sleep_enforcer.service` |
| `sleep_enforcer.service` | on trigger | runs the countdown UI |
| `sleep_enforcer_shutdown.timer` | 00:20 | starts `sleep_enforcer_shutdown.service` |
| `sleep_enforcer_shutdown.service` | on trigger | runs `systemctl poweroff` |

The countdown is purely informational. It does not shut anything down on its own
when it reaches zero, and it does not need to be running for the shutdown to
happen. The second timer is what actually powers the machine off, so killing the
UI buys you nothing. The **SHUT DOWN NOW** button just triggers the poweroff
early.

Timers are split from services because systemd keeps scheduling and execution in
separate unit types: a `.timer` knows *when*, a `.service` knows *what*. You
enable the timers; the services are pulled in on each trigger.

## Requirements

- Linux with systemd
- Rust 1.92+, required by eframe 0.35. Edition 2024 on its own only needs 1.85,
  so the graphics stack is what sets the floor.
- A graphical session. Built and tested on GNOME/X11.

## Build and install

```bash
cargo build --release
install -Dm755 target/release/sleep_enforcer ~/.local/bin/sleep_enforcer
```

Then install the units:

```bash
mkdir -p ~/.config/systemd/user
cp systemd/*.service systemd/*.timer ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now sleep_enforcer.timer sleep_enforcer_shutdown.timer
```

Confirm both timers are scheduled:

```bash
systemctl --user list-timers
```

`cargo build` alone does not update what systemd runs. Re-run the `install`
command after every rebuild.

## Changing the schedule

The shutdown time lives in two places that must agree:

1. `SHUTDOWN_HOUR` / `SHUTDOWN_MINUTE` in `src/main.rs`, which is what the
   countdown counts toward
2. `OnCalendar=` in `systemd/sleep_enforcer_shutdown.timer`, which is what
   actually powers the machine off

`OnCalendar=` in `sleep_enforcer.timer` controls how early the warning appears.
Currently the UI opens at 00:15 and counts down five minutes to a 00:20
shutdown.

If the constant and the timer drift apart, the countdown silently displays the
wrong number. Because the app rolls to the next day once the target time has
passed, a target earlier than the launch time shows a countdown of roughly 24
hours instead of a few minutes.

After editing the constant in `src/main.rs`:

```bash
cargo build --release
install -Dm755 target/release/sleep_enforcer ~/.local/bin/sleep_enforcer
```

After editing a unit file under `systemd/`, copy it over the installed one.
Editing the repo copy alone changes nothing, because systemd only ever reads
`~/.config/systemd/user/`:

```bash
cp systemd/*.service systemd/*.timer ~/.config/systemd/user/
systemctl --user daemon-reload
```

## Demo mode

```bash
sleep_enforcer --demo
```

Counts down five minutes from launch, labels itself clearly, and disables the
shutdown button. Useful for checking the layout without risking a poweroff. It
is still fullscreen and still has no close button, so exit it from another TTY
or over SSH with `pkill sleep_enforcer`.

## Testing the real path

Rather than waiting for midnight, point both timers a few minutes ahead. Edit the
copies under `systemd/`, reinstall them, then reload:

```bash
cp systemd/*.timer ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user list-timers
```

Editing through `systemctl --user edit --full` works too, but it writes only to
`~/.config/systemd/user/` and leaves the repo copies behind, so the two silently
diverge. Prefer editing the repo and copying.

Remember to put the times back afterwards, by the same route.

To check only that the UI launches correctly under systemd, without touching the
schedule:

```bash
systemctl --user start sleep_enforcer.service
```

This runs the real UI, not demo mode, so the **SHUT DOWN NOW** button is live.
The window is fullscreen with no close button, so line up a way out first. From
another TTY or over SSH:

```bash
systemctl --user stop sleep_enforcer.service
```

## Troubleshooting

```bash
journalctl --user -u sleep_enforcer.service -n 50
```

**`status=203/EXEC`** means systemd could not execute the binary. Usually it was
never installed, or the name at `~/.local/bin/sleep_enforcer` does not match
`ExecStart=` in the service file.

**The window never appears, but the machine still shuts down.** The two timers
are independent, so the poweroff fires whether or not the UI started. Check the
journal for the countdown service.

**The window fails to open from systemd but works from a terminal.** The systemd
user manager needs the display variables in its environment. Check with:

```bash
systemctl --user show-environment | grep -E 'DISPLAY|WAYLAND'
```

GNOME imports these automatically. If yours does not, add
`systemctl --user import-environment DISPLAY XAUTHORITY` to your session
startup.

**Poweroff does nothing.** Powering off from a user unit goes through polkit,
which normally permits it for an active local session. If it is denied, either
install the shutdown unit as a system unit or add a polkit rule.

## Uninstall

```bash
systemctl --user disable --now sleep_enforcer.timer sleep_enforcer_shutdown.timer
systemctl --user stop sleep_enforcer.service
rm ~/.config/systemd/user/sleep_enforcer*.{service,timer}
systemctl --user daemon-reload
rm ~/.local/bin/sleep_enforcer
```
