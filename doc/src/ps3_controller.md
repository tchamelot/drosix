# PS3 controller

Drosix offers the possibility to control the drone with a Sony PS3 sixaxis controller.
The controller connects to the drone through bluetooth.

## Setup the bluetooth stack on the beaglebone blue

In buildroot, the bluetooth subsystem should be activated in the Linux kernel.
(networking support/bluetooth subsystem support).
The system configuration for `/dev` management should be changed to use `udev`.
Finally, the bluez library is required with the sixaxis plug-in and the deprecated tools enabled.

To enable the bluetooth controller on the beaglebone the pin accessible in  `/sys/class/leds/wl18xx_bt_en` should be turned on (set brightness to 1).
Then, the controller should be attached to the bluetooth stack.
The controller is connected to the UART 3 uses the texas protocol at 300000 bauds.
```
hciattach /dev/ttyS3 texas 300000
```

Then, the bluetooth deamon can be started.
Note that the executable is not in the path but rather in `/usr/libexec/bluetooth/bluetoothd`.

A custom init script is used to automatically start the bluetooth on boot.
The script should be run after dbus because bluetoothd rely on it.

## Connecting the sixaxis controller

To connect the controller open a shell on drosix and start a new `bluetoothctl` shell.
Then connect the sixaxis to the beaglebone blue through USB.
A message will ask you to authorize a service, type yes.
Your controller is now paired and you can remove the USB cable.
No manipulation will be required for the following boot.
