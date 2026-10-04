# TODO

## Infrastructure & Provisioning (Raspberry Pi Zero)
- [ ] Setup Alpine Linux sysroot environment using `linux-rpi` kernel.
  - *Preference: Enforce static binaries.*
- [ ] Write system installation script in just:
  - Install custom software stack.
  - Pull and build system dependencies.
  - Manage runtime requirements.
  - Package final image into a `.img` or `.tar.(zst/xz)` archive for Pi Imager or just stright dd/pv.

## Software (`Ankle`)
- [ ] Create `Ankle` core service.
- [ ] Implement dual authentication support:
  - [ ] DLN authentication.
  - [ ] Internal authentication.
- [ ] Bind WebUI to an internal Unix socket to forward authentication requests.
  - *Research if this is even possible*
### Description of what the ankle is:
The Ankle is going to be the sole program which handles both remote IoT requests thru the dln infra AND also handles Remote Controller Pairing and overall remote controller logic.
Ankle dirrectly interacts with output devices and provides an interface for DLN clients and remotes to be the inputs to those devices.
Ankle will connect to a live grpc on DLN to create a FMA (Face-My-Ankle) socket, this socket gets special Messages and yeah.

Development of the `Face` software will exist on the [devinlittle-net](https://github.com/devinlittle/devinlittle-net) repository

## Software (WebUI)
- [ ] Create Achilles
- [ ] Add network management features:
  - [ ] Scan and connect to Wi-Fi networks.
  - [ ] Save Wi-Fi credentials locally.

## Hardware & Physical Design
- [ ] Design 3D printable enclosure for the Raspberry Pi Zero and subsequent ankle monitor.

## Bill of Materials / Purchasing List
# For the Ankle
- [ ] [RPi Zero2W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/)
- [ ] [A]()

# For remote
- [ ] [ESP32-C3 Adafruit Board](https://www.adafruit.com/product/5787)

# Misc
- [ ] [USB-C To UART](https://www.adafruit.com/product/4364)

