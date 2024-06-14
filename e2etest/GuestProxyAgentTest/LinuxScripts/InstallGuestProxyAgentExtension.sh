#!/bin/bash

# Copyright (c) Microsoft Corporation
# SPDX-License-Identifier: MIT

zipFile=$devExtensionSas   

echo "Starting install guest proxy agent extension script" 
timeout=300
elpased=0
echo "Get Extension Folder and Version"
while :; do
    directories=$(find /var/lib/waagent -type d -name '*Microsoft.CPlat.ProxyAgent.ProxyAgentLinux*')
    if [ $(echo "$directories" | wc -l) -eq 1 ]; then
        for dir in $directories; do 
            PIRExtensionFolderPath=$dir
            echo "PIR extension folder path" $PIRExtensionFolderPath
            PIRExtensionFolderZip="${PIRExtensionFolderPath//-/__}.zip"
            echo "PIRExtensionFolderZip:$PIRExtensionFolderZip"
        done 
        break
    fi
    ((elapsed += interval))
    if [[ $elapsed -ge $timeout ]]; then
        echo "Timeout reached. Exiting the loop."
        break
    fi
    sleep 5
done 
PIRExtensionVersion=$(echo "$PIRExtensionFolderPath" | grep -oP '(\d+\.\d+\.\d+)$')
echo "PIRExtensionVersion=$PIRExtensionVersion"
os=$(hostnamectl | grep "Operating System")
echo "os=$os"
if [[ $os == *"Ubuntu"* ]]; then 
    proxyAgentVersion="$(eval "$PIRExtensionFolderPath/ProxyAgent/ProxyAgent/GuestProxyAgent.exe --version")"
else
    proxyAgentVersion="$(eval "$PIRExtensionFolderPath/ProxyAgent/ProxyAgent/azure-proxy-agent --version")"
fi
echo "proxy agent version: $proxyAgentVersion"
statusFolder=$(find "$PIRExtensionFolderPath" -type d -name 'status')
echo "Status Directory: $statusFolder"
echo "Delete status file of PIR version" 
rm -rf $statusFolder/*

echo "detecting os and installing jq" #TODO: needs to be revisited if we support other distros
os=$(hostnamectl | grep "Operating System")
echo "os=$os"
if [[ $os == *"Ubuntu"* ]]; then
    for  i in {1..3}; do
        echo "start installing jq via apt-get $i"
        sudo apt update
        sudo apt-get install -y jq
        sleep 10
        install=$(apt list --installed jq)
        echo "install=$install"
        if [[ $install == *"jq"* ]]; then
            echo "jq installed successfully"
            break
        fi
    done
else
    for  i in {1..3}; do
        echo "start installing jq via yum $i"
        sudo yum -y install jq
        sleep 10
        install=$(yum list --installed jq)
        echo "install=$install"
        if [[ $install == *"jq"* ]]; then
            echo "jq installed successfully"
            break
        fi
    done
fi

echo "Check that status file is success with 5 minute timeout"
statusFile=$(ls $statusFolder/*.status)
echo "statusFile=$statusFile"
timeout=300
elpased=0
echo "Contents of status file:"
cat "$statusFile"
while :; do 
    extensionStatus=$(cat "$statusFile" | jq -r '.[0].status.status')
    if [[ "$extensionStatus" == "success" ]]; then
        echo "The status is success."
        break
    fi
    ((elapsed += interval))
    if [[ $elapsed -ge $timeout ]]; then
        echo "Timeout reached. Exiting the loop."
        break
    fi
    sleep 5
done

echo "Check that process ProxyAgentExt is running"
processId=$(pgrep ProxyAgentExt)
echo "processId=$processId"
if [ -z "$processId" ]; then
    echo "Process ProxyAgentExt is not running"
else 
    echo "Process ProxyAgentExt is running"
fi

echo "Delete PIR extension zip"
rm -rf $PIRExtensionFolderZip
echo "Delete PIR extension folder"
rm -rf $PIRExtensionFolder

decodedUrl=$(echo $zipFile | base64 -d)
curl -L -o $PIRExtensionFolderPathZip "$decodedUrl"
ls -l $PIRExtensionFolderPath

echo "Get PID of ProxyAgentExt and kill pidof"
pidof ProxyAgentExt | xargs kill -9

echo "Delete status file inside status folder"
rm -rf $statusFolder/*