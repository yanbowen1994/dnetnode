VAGRANTFILE_API_VERSION = "2"

Vagrant.configure(VAGRANTFILE_API_VERSION) do |config|
  config.vm.box = "generic/ubuntu1804"
  config.vm.provision "shell", inline: "apt-get --assume-yes install g++"
  config.vm.provision "shell", inline: "apt-get --assume-yes install python"
  config.vm.provision "shell", inline: "apt-get --assume-yes install perl"
  config.vm.provision "shell", inline: "apt-get --assume-yes install make"
  config.vm.provision "shell", inline: "apt-get --assume-yes install curl"
  config.vm.provision "shell", inline: "apt-get --assume-yes install git"
  config.vm.provision "shell", inline: "apt-get --assume-yes install pkg-config"
  config.vm.provision "shell", inline: "apt-get --assume-yes install libssl-dev"
  config.vm.provision "shell", path: "dnet_devup.sh"
  config.vm.synced_folder "../", "/host"


  # Provider specific settings.
  config.vm.provider "virtualbox" do |v|
    v.memory = 1024
    v.cpus = 2
  end
end