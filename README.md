# Hydro Email 

See [GettingStarted](docs/GettingStarted.md) for instructions on how to get started developing and testing the code.

And [Credentials](docs/Credentials.md) for help setting up credentials.

# Notes

* This uses DKIM verification from our fork of Cloudflare's DKIM library: https://github.com/Lay3rLabs/dkim-wasi
* `wasi:tls` is not on wa.dev, so we just clone our wit repo, but `wkg` is still used to fetch dependencies
