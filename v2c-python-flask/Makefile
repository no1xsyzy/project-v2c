.PHONY: all
all: poetry_install

poetry_install: pyproject.toml poetry.lock
	poetry install

.PHONY: run
run: poetry_install
	poetry run flask run -h127.0.0.1 -p8423
