#!/bin/bash

images=`docker image ls|tail -n +2|awk '/none/ {print $3}'`

echo "all images to delete - ${images}"

for image in ${images}
do
        echo "processing ${image}.."
        docker rmi ${image}
done

echo "Remaining images are as follows;"
docker image ls

echo "done."
