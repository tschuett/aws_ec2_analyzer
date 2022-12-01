#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""Utility for EC2."""

import botocore
import boto3
import pprint
import sys
from typing import Dict


REGIONS = ["us-east-1", "us-east-2", "us-west-2"]

INSTANCES = ["c6in.32xlarge", "hpc6id.32xlarge", "hpc6a.48xlarge", "c7gn.16xlarge",
             "r7iz.32xlarge", "c6gn.16xlarge", "hpc7g.16xlarge", "c6gn.16xlarge",
             "trn1.32xlarge"]


def describe_instance(instance: str) -> Dict:
    """Call boto3 to describe one instance."""
    for region in REGIONS:
        session = boto3.Session(region_name=region)
        client = session.client('ec2')
        try:
            result = client.describe_instance_types(InstanceTypes=[instance])
            return result
        except botocore.exceptions.ClientError as error:
            continue
    print(f"{instance} not found")
    return dict()


def print_instance(instance_type: str):
    """Print one instance."""
    result = describe_instance(instance_type)
    if result == dict():
        return 1

    pp = pprint.PrettyPrinter(indent=4)
    pp.pprint(result["InstanceTypes"][0]["InstanceType"])
    pp.pprint(result["InstanceTypes"][0]["NetworkInfo"])


def main() -> int:
    """Invoke The main function."""
    for instance_type in ["c6gn.16xlarge", "hpc6id.32xlarge", "c7gn.16xlarge",
                          "r7iz.32xlarge", "hpc7g.16xlarge"]:
        print_instance(instance_type)

    return 0


if __name__ == '__main__':
    sys.exit(main())
