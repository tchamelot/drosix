CircuitCo BeagleBone
Texas Instuments AM335x Evaluation Module (TMDXEVM3358)

Description
===========

This configuration will build a complete image for the Beaglebone blue
tailored tu run ardupilot suite

How to build it
===============

Select the default configuration for the target:
$ make bbblue_defconfig

Optional: modify the configuration:
$ make menuconfig

Build:
$ make

Result of the build
===================
output/images/
├── am335x-boneblue.dtb
├── boot.vfat
├── MLO
├── rootfs.ext2
├── sdcard.img
├── u-boot.img
├── uEnv.txt
└── zImage

To copy the image file to the sdcard use dd:
$ dd if=output/images/sdcard.img of=/dev/XXX

Tested hardware
===============
beagleboneblue (rev. 2A)

2016, Lothar Felten <lothar.felten@gmail.com>
2019, Bruno Lelievre <bruno.lelievre@free.fr>
