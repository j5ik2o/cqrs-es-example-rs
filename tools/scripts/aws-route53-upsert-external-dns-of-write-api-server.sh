#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

#if [[ ! -e ../../env.sh ]]; then
#    echo "env.sh is not found."
#    exit 1
#fi
#
## shellcheck disable=SC2034
#OUTPUT_ENV=1
#
#source ../../env.sh

HOST_NAME="write-ceer-j5ik2o.cwtest.info"
INGRESS_NAME="write-api-server"

export AWS_PAGER=""
AWS="aws --profile ${AWS_PROFILE} --region ap-northeast-1"

DNS_NAME=$(kubectl -n ceer get ingress ${INGRESS_NAME} -o json | jq -r '.status.loadBalancer.ingress[0].hostname')
echo "DNS_NAME=$DNS_NAME"

if [[ -z "${DNS_NAME}" ]]; then
    echo "Error: DNS_NAME is empty."
    exit 1
fi

ALB_NAME=$(echo $DNS_NAME | cut -d '-' -f 1-4)
echo "ALB_NAME=$ALB_NAME"

if [[ -z "${ALB_NAME}" ]]; then
    echo "Error: ALB_NAME is empty."
    exit 1
fi

ALB_ARN=$($AWS elbv2 describe-load-balancers --names ${ALB_NAME} --query 'LoadBalancers[0].LoadBalancerArn' --output text --region ap-northeast-1)
echo "ALB_ARN=$ALB_ARN"

if [[ -z "${ALB_ARN}" ]]; then
    echo "Error: ALB_ARN is empty."
    exit 1
fi

HOSTED_ZONE_ID=$($AWS elbv2 describe-load-balancers --load-balancer-arns ${ALB_ARN} --query 'LoadBalancers[0].CanonicalHostedZoneId' --output text --region ap-northeast-1)
echo "HOSTED_ZONE_ID=$HOSTED_ZONE_ID"

if [[ -z "${HOSTED_ZONE_ID}" ]]; then
    echo "Error: HOSTED_ZONE_ID is empty."
    exit 1
fi

$AWS route53 change-resource-record-sets \
	--hosted-zone-id Z20E1ZAK4UG4IP \
	--change-batch "$(cat <<-EOF
{
  "Changes": [
    {
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "${HOST_NAME}",
        "Type": "A",
        "AliasTarget": {
          "HostedZoneId": "${HOSTED_ZONE_ID}",
          "DNSName": "${DNS_NAME}",
          "EvaluateTargetHealth": false
        }
      }
    }
  ]
}
EOF
)"

echo "External DNS Name = ${HOST_NAME}"