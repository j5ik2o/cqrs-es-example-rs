#!/usr/bin/env bash

kubectl run debug -it --image=ubuntu --rm --restart=Never -- /usr/bin/bash


# apt update
# apt install mysql-client
# mysql -u root -h aht9aa1e-ceer-mysql.cluster-ctywrcabnmgr.ap-northeast-1.rds.amazonaws.com -P 3306 -p