cargo build --target x86_64-unknown-uefi --example "$1"
rm esp/EFI/BOOT/BOOTX64.EFI
cp target/x86_64-unknown-uefi/debug/examples/$1.efi esp/EFI/BOOT/BOOTX64.EFI
rm OVMF_CODE.fd
rm OVMF_VARS.fd
cp /usr/share/ovmf/x64/OVMF_CODE.fd .
cp /usr/share/ovmf/x64/OVMF_VARS.fd .
qemu-system-x86_64 \
-machine q35 \
-drive if=pflash,format=raw,file=OVMF_CODE.fd \
-drive if=pflash,format=raw,file=OVMF_VARS.fd \
-drive format=raw,file=fat:rw:esp