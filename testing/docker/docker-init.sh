#!/bin/bash
# TODO : add ssh key checking : if the controller_key has changed, we need to rebuild
# the Docker images to embed the new key
ContainerList=$1
ModuleName=$2

echo "Init script launched..."

# First, we check if the situation is clear. If something went wrong before the cleanup script
# had a chance to run last time, we will have some residual datas that we need to clean.
# If containers-info.json is not empty, run the cleanup script first.
if [ -s $ContainerList ]
then
    ./docker-scripts/docker-cleanup.sh $ContainerList $ModuleName
fi

# To have a passwordless root access, we need keys. If it hasn't already been done,
# let's generate keys in order to have it allowed in the containers later.
if [ ! -f controller_key ] || [ ! -f controller_key.pub ]
then
    rm -f controller_key*
    ssh-keygen -t ed25519 -f controller_key -N "" -q
fi

# Building a JSON file for containers handling and an inventory for
# the actual use of the containers

# Opening an array in the JSON file
echo "{" >> $ContainerList
echo '"containers_list": [' >> $ContainerList

# Opening the inventory file
InventoryFile="$ModuleName.yaml"
echo "---" >> $InventoryFile
echo "hosts:" >> $InventoryFile


# Here we list all the Dockerfiles available
DOCKERFILES_LIST=$(find Dockerfiles -type f -name "Dockerfile-*")

for Dockerfile in $DOCKERFILES_LIST
do
    # Building the image with an explicit name
    OsName=$(basename $Dockerfile | cut -d "-" -f 2)
    docker build -f $Dockerfile -t dux-host-$ModuleName-$OsName:latest .

    # Running a container based on this image and retrieving informations on it
    ContainerID=$(docker run -d dux-host-$ModuleName-$OsName)
    ContainerIP=$(docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $ContainerID)
    ContainerPubKey=$(ssh-keyscan $ContainerIP)

    # Filling the JSON file with container's informations
    echo {\"container_name\" : \"dux-host-$ModuleName-$OsName\", >> $ContainerList
    echo \"container_id\" : \"$ContainerID\", >> $ContainerList
    echo \"container_ip\" : \"$ContainerIP\", >> $ContainerList

    # Having the container's key allowed on the host to avoid the usual StrictHostKeyChecking issues
    if [ -e ~/.ssh/known_hosts ]
    then
        echo $ContainerPubKey >> ~/.ssh/known_hosts
    else
        touch ~/.ssh/known_hosts
        echo $ContainerPubKey >> ~/.ssh/known_hosts
    fi
    
    # Filling the inventory file
    echo "  - $ContainerIP" >> $InventoryFile

done

# The last line ends with "}," (invalid JSON syntax) which needs to be changed to "}".
sed -i '$ s@},@}@g' $ContainerList

# Closing the array of the JSON file
echo "]" >> $ContainerList
echo "}" >> $ContainerList