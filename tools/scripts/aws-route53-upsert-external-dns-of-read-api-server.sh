#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

if [[ ! -e ../../env.sh ]]; then
    echo "env.sh is not found."
    exit 1
fi

# shellcheck disable=SC2034
OUTPUT_ENV=1

source ../../env.sh

HOST_NAME="read-ceer-j5ik2o.cwtest.info"
INGRESS_NAME="read-api-server"

export AWS_PAGER=""
AWS="aws --profile ${AWS_PROFILE} --region ap-northeast-1"

DNS_NAME=$(kubectl -n ceer get ingress ${INGRESS_NAME} -o json | jq -r '.status.loadBalancer.ingress[0].hostname')
echo "DNS_NAME=$DNS_NAME"

ALB_NAME=$(echo $DNS_NAME | cut -d '-' -f 1-4)
echo "ALB_NAME=$ALB_NAME"

ALB_ARN=$($AWS elbv2 describe-load-balancers --names ${ALB_NAME} --query 'LoadBalancers[0].LoadBalancerArn' --output text --region ap-northeast-1)
echo "ALB_ARN=$ALB_ARN"

HOSTED_ZONE_ID=$($AWS elbv2 describe-load-balancers --load-balancer-arns ${ALB_ARN} --query 'LoadBalancers[0].CanonicalHostedZoneId' --output text --region ap-northeast-1)
echo "HOSTED_ZONE_ID=$HOSTED_ZONE_ID"

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