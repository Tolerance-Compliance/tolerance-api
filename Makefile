.PHONY: build stop rebuild logs

build:
	docker compose up -d --build

stop:
	docker compose down --remove-orphans
	-lsof -ti :3000 | xargs kill -9 2>/dev/null

rebuild:
	$(MAKE) stop
	docker compose up -d --build

logs:
	docker compose logs -f
