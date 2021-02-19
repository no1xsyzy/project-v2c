import yaml
import requests
import converted_data


def fetch(path):
    g = requests.get("http://localhost:8423/v2rayn_to_clash", params=[('from', f"http://localhost:8434/{path}")])
    if g.status_code != 200:
        raise
    return yaml.safe_load(g.content)


def test_check():
    assert fetch("one") == converted_data.one
    assert fetch("five") == converted_data.five
