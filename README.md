# Rust AWS DDNS

This project implements a simple alternative to DDNS services using AWS Route53
as the DNS provider. It achieves that by discovering the machine public internet
ip addresses (both IPv4 and IPv6) and then updates the RecordSets on the
configured Route53 account.

The tool can run on small devices, like raspberry pis, and automatically with
a simple cron entry.
