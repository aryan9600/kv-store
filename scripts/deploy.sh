 #!/usr/bin/env bash
container_id=""
for id in `docker ps --format "table {{.ID}}  {{.Names}}  {{.CreatedAt}}" | grep web | awk -F  "  " '{print $1}'`
do
    container_id="$id"
done
echo "found running web container: $container_id"
echo "scaling web to 2 containers"
docker-compose up -d --build --no-deps --scale web=2 --no-recreate web
sleep 10
echo "killing container $container_id"
docker kill -s SIGTERM $container_id
sleep 1
echo "removing container $container_id"
docker rm -f $container_id
sleep 1
echo "scaling back down to one container"
docker-compose up -d --no-deps --scale web=1 --no-recreate web
