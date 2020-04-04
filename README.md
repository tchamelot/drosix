# Drosix

Drosix is an ugly name that I use to speak about my project of building my own 
quad copter from scratch.

## Why?

Why not!

My goal is to create a complete system using a Linux system. A drone might be
too complex to start with, but I think that I have enough experience to give it 
a try. This project is more about embedded softwares than drones.

Rust is a in my opinion a great language. I think that this kinds of project are 
good candidate to use Rust as the primary language.

## Hardware

* Controler : Beaglebone blue
* IMU : MPU9250 (integrated with the Beaglebone Blue)
* Motors : RCTimer A2830-14 750KV
* ESCs : Hobbypower ESC-30A
* Propellers : 10x4.5

## Frame

I plan to use a 450mm frame that I will build myself. This sections will be 
updated when I will actually have a complete frame. For now, I use a 3D printed 
frame that a friend designed for me.

## Software

I have not done a lot yet. I am working on MPU9250 acquisition through the 
[rust crate](https://github.com/coptrust/mpu9250) where I have a PR open. I also 
worked on controlling the ESCs using the PRU and the 
[prusst](https://github.com/sbarral/prusst) crate.

The `.cargo` folder contains files which allows you to use `cargo run` directly 
on your work station assuming an USB connection between it and the Beaglebone 
blue. Moreover, the Beagleblone should run the distribution built with buildroot 
and br2-drosix-tree. This image does not offer any security as the private key 
which allows to connect as root on the Beagleblone is public in `.cargo`.
