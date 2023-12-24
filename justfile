#!/usr/bin/env -S just --justfile

docker-rebuild:
	docker build --force-rm -t chat_server ./chat-server

docker-run:
	docker run -d --rm -e POSTGRES_PASSWORD=pw --name=pg -p 5432:5432 chat_server

# vim: set ft=make ts=2 sw=2 sts=2 :
