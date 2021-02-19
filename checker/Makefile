.PHONY: all
all:
	poetry install

.PHONY: serve
serve:
	FLASK_APP=server.py poetry run flask run -h127.0.0.1 -p8434

.PHONY: check
check:
	poetry run pytest
