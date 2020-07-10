# Buildroot

## Device Tree

The device tree is a way to represent hardware in the Linux kernel. It allows 
the kernel to be independent from the hardware by loading the right device tree 
blob at boot time. For the Beaglebone ecosystem, the device tree may enable some 
features such as Bluetooth, I2C, SPI or the PRU subsystems.

In order to make those configuration modular, the device tree support what is 
called overlay. Overlays are additional files that can be built alone. Overlay 
will be selected at boot time to overwrite some part of the device tree blob and 
enable (or disable) specific features. For the Beaglebone overlays are handled 
by the `capemgr` which is implemented both in Beaglebone's U-boot and Linux
kernel. The `capemgr` is not standard. In the U-boot's configuration, namely 
`uEnv.txt`, you can select which overlay to add to your system.

The real problem start when you want to use Buildroot. You might not want to 
support overlay but rather have a static device tree blob with the final 
configuration. In order to do that, you have to patch the device tree sources. 
For the Beaglebone Blue, the files are:

'''
linux/path/arch/arm/boot/dts/am335x-boneblue.dts
linux/path/arch/arm/boot/dts/am33xx.dtsi
'''

The device tree is organised as the Linux file system. `/` is the root. The tree 
as nodes which have child nodes.

'''
/parent/
    child1/
        leaf
    child2/
        leaf1
'''

Overlays specify the path to the node to modify. Once you have identify the 
overlay you want to add, you need to find the path it modify in the device tree 
sources. Then, you can patch the sources by overwriting the sources with the 
configuration. This is what would have happened if the overlay had been enabled 
at boot time.

Now that the device tree is modified, we can create a patch for Linux.

'''
diff -Naur path/to/base/linux path/to/modify/linux > modify-dts.patch
'''

Then place the patch file in `path/to/buildroot/linux`. The patch will 
automatically be applied.

In my case the two overlay that I added were:

* AM335X-PRU-UIO-00A0.dts
* BB-ADC-00A0.dts
