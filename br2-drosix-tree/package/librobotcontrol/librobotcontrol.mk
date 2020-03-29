################################################################################
#
# librobotcontrol
#
################################################################################

LIBROBOTCONTROL_VERSION = 2d2467017519ab2b48cd20715f7449de3132cfb9
LIBROBOTCONTROL_SITE = $(call github,StrawsonDesign,librobotcontrol,$(LIBROBOTCONTROL_VERSION))
LIBROBOTCONTROL_LICENSE = MIT
LIBROBOTCONTROL_LICENSE_FILES = LICENCE
LIBROBOTCONTROL_INSTALL_STAGING = YES

define LIBROBOTCONTROL_BUILD_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) $(TARGET_CONFIGURE_OPTS) -C $(@D)/library all
endef

define LIBROBOTCONTROL_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 $(@D)/library/lib/librobotcontrol.so.1.0.4 $(TARGET_DIR)/usr/lib/librobotcontrol.so.1
	ln -sf librobotcontrol.so.1 $(TARGET_DIR)/usr/lib/librobotcontrol.so
	$(INSTALL) -d -m 0777 $(TARGET_DIR)/var/lib/robotcontrol
endef

define LIBROBOTCONTROL_INSTALL_STAGING_CMDS
    cp -r --no-dereference --preserve=mode,links $(@D)/library/include/* $(STAGING_DIR)/usr/include
endef

$(eval $(generic-package))
