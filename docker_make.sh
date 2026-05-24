#!/bin/bash -e
cd bin/web
    npm run build
cd ..
docker build -t raycanary-devenv -f tools/devenv.dockerfile .
echo ' build!'
docker run --user $UID:$GID -v ./:/workdir -w /workdir -it raycanary-devenv sh -c 'cargo build --release --target="armv7-unknown-linux-musleabihf"'
adb shell '/bin/rootshell -c "/etc/init.d/raycanary_daemon stop"'
adb push target/armv7-unknown-linux-musleabihf/release/raycanary-daemon /data/raycanary/raycanary-daemon
echo "rebooting the device..."
adb shell '/bin/rootshell -c "reboot"'
