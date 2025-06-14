# Create directories for the ESP
mkdir -p esp/EFI/BOOT

# Copy your bootloader
cp target/x86_64-unknown-uefi/debug/rustos-bootloader.efi esp/EFI/BOOT/BOOTX64.EFI

# Copy the test kernel
cp test-files/kernel.elf esp/kernel.elf
