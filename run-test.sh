#!bash

set -e

echo "Prerequisites: you first have to create the tap device, like this :"
echo "sudo ip tuntap add tapfc1 mode tap"
echo "Elsewhere run :"
echo "rm /tmp/firecracker.socket && firecracker"
echo ""

echo Running ArnOs on Firecracker...

curl --unix-socket /tmp/firecracker.socket -i \
    -X PUT 'http://localhost/boot-source'   \
    -H 'Accept: application/json'           \
    -H 'Content-Type: application/json'     \
    -d '{
        "kernel_image_path": "/home/arnaud/repos/third/firecracker/hello-vmlinux.bin",
        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
    }'

curl --unix-socket /tmp/firecracker.socket -i \
    -X PUT 'http://localhost/drives/rootfs' \
    -H 'Accept: application/json'           \
    -H 'Content-Type: application/json'     \
    -d '{
        "drive_id": "rootfs",
        "path_on_host": "/home/arnaud/repos/third/firecracker/hello-rootfs.ext4",
        "is_root_device": true,
        "is_read_only": false
    }'

curl --unix-socket /tmp/firecracker.socket -i \
    -X PUT 'http://localhost/logger'       \
    -H  'Accept: application/json'          \
    -H  'Content-Type: application/json'    \
    -d '{
        "log_fifo": "/dev/stdout",
        "metrics_fifo": "/dev/null"
    }'

curl --unix-socket /tmp/firecracker.socket -i \
    -X PUT 'http://localhost/network-interfaces/iface_1'       \
    -H  'Accept: application/json'          \
    -H  'Content-Type: application/json'    \
    -d '{
        "iface_id": "iface_1",
        "host_dev_name": "tapfc1",
        "guest_mac": "06:00:c0:a8:34:02"
    }'

curl --unix-socket /tmp/firecracker.socket -i -X GET 'http://localhost/machine-config'

curl --unix-socket /tmp/firecracker.socket -i \
    -X PUT 'http://localhost/actions'       \
    -H  'Accept: application/json'          \
    -H  'Content-Type: application/json'    \
    -d '{
        "action_type": "InstanceStart"
    }'

echo all commands sent