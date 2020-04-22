# Buildroot external tree for Drosix

This folder contains resources to build a custom Linux distribution for a 
Beaglebone Blue.

## Buildroot

> Buildroot is a simple, efficient and easy-to-use tool to generate embedded 
> Linux systems through cross-compilation.

## Usage

1. Install Buildroot with either:
    * Downloading and extracting the 
      [archive](https://buildroot.org/download.html)
    * Cloning the [repositopry](https://git.buildroot.net/buildroot)
2. In the Buildroot folder run `make BR2_EXTERNAL=/path/to/drosix/br2-drosix-tree 
   beagleboneblue_defconfig`
3. Run `make all` to build the sd card image
4. Run `dd if=/path/to/buildroot/output/images/sdcard.image of=/path/to/sdcard bs=4M` 
   to install the image on the sd card
5. Boot the Beaglebone Blue on the sd card. You should see a new network interface 
   which correspond to an Ethernet over USB interface. Configure it with the 
   address 192.168.7.1/24. Then you should be able to connect to the beaglebone 
   with `ssh root@192.168.7.2` and the password `toor`

## Warning

This image does not offer any security as the private key 
which allows to connect as root on the Beagleblone is public in `.cargo`.
