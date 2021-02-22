# Project V2C

## Abstract

This project is to implement the same requirement in different languages and/or frameworks.

They are used for comparing difficulties in these languages.

## Target

1. Start a Webserver.
2. Serve at `http://127.0.0.1:8423/v2rayn_to_clash?from={upstream_url}`
3. On each request, fetch `upstream_url` as V2rayN subscription, translate it into clash outfile, and response with it.
4. All others are undefined behavior. Check other cases? At implementer's wish.

## Existing Implementations

So far included:

* Python + Flask + Requests
* Go + gin
* Rust + tokio + hyper + reqwest
* Rust + actix-web
* Ponylang

## Licensing

GPLv3+
