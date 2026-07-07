sudo $(which dpdk-testpmd) -l 0-1 --vdev net_vhost0,iface=/tmp/vhost-user1,client=1 --single-file-segments -- --forward-mode=rxonly --stats-period=1
sudo $(which dpdk-testpmd) -l 2-3 --file-prefix=pmd0 --vdev net_virtio_user0,path=/tmp/vhost-user1,server=1 --single-file-segments -- --txpkts=60 --forward-mode=txonly --stats-period=1

sudo unlink /tmp/vhost-user1