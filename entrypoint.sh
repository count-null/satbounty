#!/bin/bash

# Required environment variables
# export SATBOUNTY_DB_URL=
# export SATBOUNTY_ADMIN_USERNAME=
# export SATBOUNTY_ADMIN_PASSWORD=
# export SATBOUNTY_LND_HOST=
# export SATBOUNTY_LND_PORT=
# export SATBOUNTY_LND_TLS_CERT_PATH=
# export SATBOUNTY_LND_MACAROON_PATH=

# Generate a secret
export ROCKET_SECRET_KEY=$(openssl rand -base64 32)

# if lnd enabled, attempt to connect
exec satbounty

