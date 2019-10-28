include $(TOPDIR)/rules.mk

PKG_NAME:=dnet
PKG_VERSION:=20180601
PKG_RELEASE:=3

PKG_BUILD_DIR:=$(BUILD_DIR)/$(PKG_NAME)

include $(INCLUDE_DIR)/package.mk

define Package/dnet
  SECTION:=net
  CATEGORY:=Network
  TITLE:=ConfigAgent and CloudAgent Daemon
  URL:=http://pptpclient.sourceforge.net/
endef


define Build/Prepare
endef


define Build/Compile
endef

define Package/dnet/install
	$(INSTALL_DIR) $(1)/usr/local/opt/dnet
	$(INSTALL_BIN) ./dnet/settings.toml $(1)/usr/local/opt/dnet/settings.toml
	$(INSTALL_BIN) ./dnet/dnet-daemon $(1)/usr/local/opt/dnet/dnet-daemon

	$(INSTALL_DIR) $(1)/usr/local/opt/dnet/tinc
	$(INSTALL_BIN) ./dnet/tinc/tinc-report $(1)/usr/local/opt/dnet/tinc/tinc-report

	install -d -m0755 $(1)/www
	install -d -m0755 $(1)/www/cgi-bin
	$(CP) ./dnet/dnet-cgi $(1)/www/cgi-bin/vpnctrl.cgi
	install -d -m0755 $(1)/etc
	install -d -m0755 $(1)/etc/init.d
	$(CP) ./dnet/init.d $(1)/etc/init.d/dnet-daemon
endef


$(eval $(call BuildPackage,dnet))
