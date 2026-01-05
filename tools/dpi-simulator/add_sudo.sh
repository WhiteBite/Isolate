#!/bin/bash
echo "dpi ALL=(ALL) NOPASSWD:ALL" | sudo tee /etc/sudoers.d/dpi
sudo chmod 440 /etc/sudoers.d/dpi
