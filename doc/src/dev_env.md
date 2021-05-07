# Development environment

The drosix project uses [Docker](https://www.docker.com/) to create a container for development purpose.
The container is built from `docker/Dockerfile`.
It contains all the required packages to create the Linux distribution from [buildroot](https://git.buildroot.net/buildroot).
Additionally, it provides the rust toolchain configured to generate directly for the beaglebone blue.

## Buildroot

The container provides buildroot 2020.08 in `/home/worker/buildroot`.
To create the Linux distribution, start the container by running

```
$ make docker-run
```
You should now have a shell open in the container and you can run
```
worker@drosix_dev:~$ make O=/home/worker/output -C buildroot beagleboneblue_defconfig
```
to setup the buildroot environment in `/home/worker/output`.

Note that this folder is a Docker volume and will be saved between several run of the `make docker-run` commands.
This permits to avoid rebuilding the entire each time you restart the container.

Once the environment is set up, you can go in the output folder and run `make` to
build the distribution or `make nconfig` to modify the configuration.
The resulting images will be available in the `images` folder on your host
system.

## Rust

To build rust binaries, start the docker container

```
$ make docker-run
```
and then go to the drosix folder and use cargo to build the binaries
```
worker@drosix_dev:~$ cd drosix
worker@drosix_dev:~$ cargo build
```

The resulting binaries can be found on the host in `target/armv7-unknown-linux-gnueabihf/release`.

## Webassmebly

TODO

## Docker notes

### Container user

By default, docker container only have a single user which is root.
However, a non root user is required to use buildroot.
This is why a custom user is created in the Dockerfile to run buildroot in the recommended conditions.

### Shared volumes

The docker container is used to build artifact in a configuration-less environment.
The container file system is completely isolated from the host system.
This is why this project uses shared volumes between the host and the container.
The following folders from your host are mounted in the container:

- drosix:/home/worker/drosix
- drosix/images:/home/worker/output/images

With this configuration, it is possible to work on drosix on the host system.
Then it is not required to rebuild or restart the container to rebuild an artifact.
Additionally, the buildroot artifacts are directly available on the host system in the `images` folder.

However, those folders are shared by your user on the host system and `worker` in the container.
To avoid permissions issue, the `worker` user is created using the current user and group ids.

### Cache volumes

A Docker container does not provide any persistent data storage between different invocations.
Yet buildroot creates artifacts that should not leak into your host system but should remain between container invocation.
Docker provides volume that are folder shared with the host but entirely managed by docker.
A volume provide a way to save data when a container is stopped.
The following volumes are mounted in the container at start up by the make invocation:

- drosix\_dl:/home/worker/dl is used to cache the package downloads by buildroot
- drosix\_output:/home/worker/output is used to cache the buildroot artifacts (toolchain, sysroot, ...)

There is no way to control the permissions of those volumes in the container.
They are owned by the root user.
To make them accessible to `worker` a custom entry point script is run when the compiler start.
It ensures that all the folders in `/home/worker` are owned by `worker` by running `chown`.

### Rust environment

Rust needs an hint to find the linker when cross compiling.
This is adding those lines into `$HOME/.cargo/config`

```
[target.armv7-unknown-linux-gnueabihf]
linker = "/home/worker/output/host/bin/arm-drosix-linux-gnueabihf-gcc"
```

Additionally, several environment variables are required to cross compile the rust code to armv7.

- TARGET\_CC: the path to the armv7 compiler
- SYSROOT: the path to the drosix sysroot (buildroot staging directory)
- PKG\_CONFIG: path to the pkg-config executable
- PKG\_CONFIG\_ALLOW\_CROSS: allow the use of pkg-config even when cross compiling
- PKG\_CONFIG\_LIB\_DIR: the paths where to find pkg-config metadata
- PKG\_CONFIG\_SYSROOT: the sysroot that pkg-config should use

