import json
from base64 import b64encode

from flask import Flask

import original_data

app = Flask("reference-server")


def encodes_v2rayn(lst):
    return b64encode(
        "\n".join(
            ("vmess://" + b64encode(json.dumps(item).encode("utf-8")).decode('utf-8')) for item in lst
        ).encode("utf-8")
    ).decode('utf-8')


@app.route("/one")
def one():
    return encodes_v2rayn(original_data.one)


@app.route("/five")
def five():
    return encodes_v2rayn(original_data.five)
