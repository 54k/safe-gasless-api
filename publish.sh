#docker login --username= --password= registry.heroku.com
export DOCKER_DEFAULT_PLATFORM=linux/amd64
docker build --platform=linux/amd64 -t web .
docker tag web registry.heroku.com/stormy-wave-90542/web
docker push registry.heroku.com/stormy-wave-90542/web
heroku container:release web
heroku logs --tail
