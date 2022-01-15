# -*- mode: ruby -*-
# vi: set ft=ruby :
Vagrant.configure(2) do |config|
    config.vm.box = "ubuntu/hirsute64"
    config.vm.hostname = "roadguard-dev"
    config.vm.provider "virtualbox" do |v|
        v.name = "roadguard-dev"
        v.memory = 4096
        v.cpus = 2
    end
    config.vm.provision "shell", inline: <<-SHELL
    sudo apt-get update -y

    # dependencies
    sudo apt-get install -y \
        wireguard \
        resolvconf \
        build-essential
    SHELL
end