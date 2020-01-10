qemu-img create -f qcow2 disk.data 8g
qemu-system-x86_64 \
    -m 2048 \
    -machine accel="hax:kvm:hvf:tcg" \
    -drive if=virtio,media=disk,file=disk.data \
    -netdev user,id=mynet0,net=192.168.76.0/24,dhcpstart=192.168.76.9 \
    -device e1000,netdev=mynet0

qemu-system-x86_64 \
    -m 2048 \
    -machine accel="hax:kvm:hvf:tcg" \
    -drive if=virtio,media=disk,file=disk.data \
    -netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no \
    -device virtio-net-pci,netdev=mynet0



qemu-system-x86_64 \
    -m 2048 \
    -machine accel="hax:kvm:hvf:tcg" \
    -nographic \
    -serial mon:stdio \
    -rtc base=utc,clock=rt \
    -chardev socket,path=qga.sock,server,nowait,id=qga0 \
    -device virtio-serial \
    -device virtserialport,chardev=qga0,name=org.qemu.guest_agent.0 \
    -drive if=virtio,media=disk,file=disk.data \
    -netdev user,id=mynet0,net=192.168.76.0/24,dhcpstart=192.168.76.9 \
    -device e1000,netdev=mynet0
    -append "console=${CONSOLE:=ttyS0} loglevel=${LOGLEVEL:=4} printk.devkmsg=${PRINTK_DEVKMSG:=on}"



qemu-system-x86_64 \
    -m 2048 \
    -machine accel="hax:kvm:hvf:tcg" \
    -drive if=virtio,media=disk,file=disk.data \
    -netdev user,id=mynet0,net=192.168.76.0/24,dhcpstart=192.168.76.9 \
    -device e1000,netdev=mynet0

    -cdrom k3os-amd64.iso \
    -netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no \

    -nographic \
    -serial mon:stdio \
    -rtc base=utc,clock=rt \
    -chardev socket,path=$STATE_DIR/qga.sock,server,nowait,id=qga0 \
    -device virtio-serial \
    -device virtserialport,chardev=qga0,name=org.qemu.guest_agent.0 \
    -kernel $(dirname $0)/../dist/artifacts/k3os-vmlinuz-$ARCH \
    -initrd $(dirname $0)/../dist/artifacts/k3os-initrd-$ARCH \
    -drive if=ide,media=cdrom,file=$(dirname $0)/../dist/artifacts/k3os-$ARCH.iso \
    -append "console=${CONSOLE:=ttyS0} loglevel=${LOGLEVEL:=4} printk.devkmsg=${PRINTK_DEVKMSG:=on} $*"