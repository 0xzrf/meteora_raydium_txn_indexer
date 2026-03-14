#!/bin/bash

wallets=("$@")
wallets_json=$(printf '"%s",' "${wallets[@]}")
wallets_json="[${wallets_json%,}]"

curl -X POST -H "Content-Type: application/json" -d "{\"wallets\":$wallets_json}" http://localhost:3000/add_wallet