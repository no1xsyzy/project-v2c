from base64 import b64decode
from json import loads

import requests
import yaml
from flask import Flask
from flask import make_response
from flask import request


app = Flask("v2c-python-flask")


def _helper_transfer_v2rayn_to_clash_one(v2rayn):
    address = v2rayn["add"]
    port = v2rayn["port"]
    id_ = v2rayn["id"]
    alter_id = v2rayn["aid"]
    security = "auto"
    network = v2rayn["net"]
    remarks = v2rayn["ps"]
    type_ = v2rayn["type"]
    host = v2rayn["host"]
    path = v2rayn["path"]
    tls = v2rayn["tls"] == "tls"
    #  {'v': '2'} ?

    cast = {
        "name": remarks,
        "type": "vmess",
        "server": address,
        "port": port,
        "uuid": id_,
        "alterId": alter_id,
        "cipher": security,
        "skip-cert-verify": True,
        # 'servername': host,
        "network": network,
        "ws-path": path,
        "ws-headers": {
            "host": host,
        },
    }

    if tls:
        cast["tls"] = True

    return cast


def v2rayn(url):
    content = requests.get(url).content
    lines = b64decode(content).split(b"\n")
    return [loads(b64decode(line[8:])) for line in lines if line]


@app.route("/v2rayn_to_clash")
def v2rayn_to_clash():
    if from_url := request.args.get("from"):
        v2rayn_servers = v2rayn(from_url)
        clash_servers = [
            _helper_transfer_v2rayn_to_clash_one(v2rayn_server)
            for v2rayn_server in v2rayn_servers
        ]
        resp = make_response(yaml.safe_dump({"proxies": clash_servers}))
        resp.content_type = "text/yaml"
        return resp
