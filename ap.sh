sudo mkdir -p /run/systemd/network
sudo cp 05-hotspot.network /run/systemd/network/05-hotspot.network
sudo networkctl reload
sudo networkctl reconfigure wlan6

sudo iwctl device wlan6 set-property Mode ap
#other mode is station
sudo iwctl ap wlan6 start-profile Research
sudo sysctl -w net.ipv4.ip_forward=1
sudo iptables -t nat -A POSTROUTING -s 192.168.50.0/24 -o wlan0 -j MASQUERADE